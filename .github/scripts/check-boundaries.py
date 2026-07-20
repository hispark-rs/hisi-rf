#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
"""Verify the released RF facade keeps integration crates transitive."""

from __future__ import annotations

import argparse
import json
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
CONSUMER = ROOT / ".github" / "fixtures" / "ws63-consumer"
TARGET = "riscv32imfc-unknown-none-elf"
NAMED_PROFILES = {
    "wpa2-personal": "profile-wifi-wpa2-smoltcp",
    "wpa3-personal": "profile-wifi-wpa3-smoltcp",
}
HIDDEN = {
    "hisi-rf-core",
    "hisi-rf-rtos-driver",
    "hisi-rf-ws63",
    "ws63-radio-blob",
    "ws63-radio-sys",
}
FORBIDDEN_PUBLIC_TOKENS = {
    "hisi_rf_rtos_driver",
    "hisi_rf_ws63",
    "ws63_radio_blob",
    "ws63_radio_sys",
}


def metadata(manifest: Path, features: str) -> dict:
    command = [
        "cargo",
        "metadata",
        "--locked",
        "--format-version",
        "1",
        "--filter-platform",
        TARGET,
        "--manifest-path",
        str(manifest),
        "--features",
        features,
    ]
    completed = subprocess.run(
        command,
        cwd=manifest.parent,
        check=False,
        capture_output=True,
        text=True,
    )
    if completed.returncode != 0:
        raise ValueError(
            f"cargo metadata failed for {manifest.parent.name}:\n{completed.stderr.strip()}"
        )
    return json.loads(completed.stdout)


def graph(meta: dict, manifest: Path) -> tuple[dict[str, dict], dict[str, str], str]:
    packages = {package["id"]: package for package in meta["packages"]}
    resolve = meta.get("resolve")
    if resolve is None:
        raise ValueError("cargo metadata did not return a resolve graph")
    all_nodes = {node["id"]: node for node in resolve["nodes"]}
    root_manifest = manifest.resolve()
    roots = [
        package_id
        for package_id, package in packages.items()
        if Path(package["manifest_path"]).resolve() == root_manifest
    ]
    if len(roots) != 1:
        raise ValueError(
            f"expected one package for {manifest}, found {len(roots)}"
        )
    root = roots[0]

    reachable: set[str] = set()
    pending = [root]
    while pending:
        package_id = pending.pop()
        if package_id in reachable:
            continue
        reachable.add(package_id)
        pending.extend(dependency["pkg"] for dependency in all_nodes[package_id]["deps"])

    nodes = {package_id: all_nodes[package_id] for package_id in reachable}
    names = {package_id: packages[package_id]["name"] for package_id in reachable}
    return nodes, names, root


def direct_names(nodes: dict[str, dict], names: dict[str, str], package_id: str) -> set[str]:
    return {names[dependency["pkg"]] for dependency in nodes[package_id]["deps"]}


def unique_id(names: dict[str, str], package_name: str) -> str:
    matches = [package_id for package_id, name in names.items() if name == package_name]
    if len(matches) != 1:
        raise ValueError(
            f"expected exactly one {package_name!r} package, found {len(matches)}"
        )
    return matches[0]


def require_edge(
    nodes: dict[str, dict], names: dict[str, str], parent: str, child: str
) -> None:
    parent_id = unique_id(names, parent)
    if child not in direct_names(nodes, names, parent_id):
        raise ValueError(f"missing required dependency edge: {parent} -> {child}")


def check_source(profile: str) -> None:
    meta = metadata(
        ROOT / "Cargo.toml",
        f"hisi-rf/chip-ws63,hisi-rf/{NAMED_PROFILES[profile]}",
    )
    nodes, names, root = graph(meta, ROOT / "Cargo.toml")
    if names[root] != "hisi-rf":
        raise ValueError(f"source metadata root is {names[root]!r}, expected 'hisi-rf'")

    root_deps = direct_names(nodes, names, root)
    leaked = sorted((HIDDEN - {"hisi-rf-core", "hisi-rf-ws63"}) & root_deps)
    if leaked:
        raise ValueError("facade directly depends on hidden implementation crates: " + ", ".join(leaked))

    require_edge(nodes, names, "hisi-rf", "hisi-rf-core")
    require_edge(nodes, names, "hisi-rf", "hisi-rf-ws63")
    require_edge(nodes, names, "hisi-rf-ws63", "hisi-rf-rtos-driver")
    require_edge(nodes, names, "hisi-rf-ws63", "ws63-radio-sys")
    require_edge(nodes, names, "ws63-radio-sys", "ws63-radio-blob")

    if "ws63-rf-rs" in names.values():
        raise ValueError("legacy ws63-rf-rs leaked into the facade dependency graph")

    for package_name in HIDDEN | {"hisi-rf"}:
        unique_id(names, package_name)


def check_consumer(profile: str) -> None:
    meta = metadata(
        CONSUMER / "Cargo.toml",
        f"hisi-rf/{profile}",
    )
    nodes, names, root = graph(meta, CONSUMER / "Cargo.toml")
    if names[root] != "hisi-rf-ws63-external-consumer":
        raise ValueError(f"consumer metadata root is {names[root]!r}")

    root_deps = direct_names(nodes, names, root)
    hidden = sorted(root_deps & HIDDEN)
    if hidden:
        raise ValueError("consumer directly depends on hidden RF crates: " + ", ".join(hidden))
    if "hisi-rf" not in root_deps:
        raise ValueError("consumer does not depend directly on the hisi-rf facade")

    packages = {package["name"]: package for package in meta["packages"]}
    for package_name in HIDDEN | {"hisi-rf"}:
        package = packages.get(package_name)
        if package is None:
            raise ValueError(f"released consumer graph is missing {package_name}")
        source = package.get("source") or ""
        if not source.startswith("registry+"):
            raise ValueError(
                f"released consumer resolved {package_name} outside a registry: {source or 'path'}"
            )

    require_edge(nodes, names, "hisi-rf", "hisi-rf-ws63")
    require_edge(nodes, names, "hisi-rf-ws63", "ws63-radio-sys")
    require_edge(nodes, names, "ws63-radio-sys", "ws63-radio-blob")


def check_rustdoc(doc_dir: Path) -> None:
    if not doc_dir.is_dir():
        raise ValueError(f"rustdoc output does not exist: {doc_dir}")
    leaks: list[str] = []
    for path in sorted(doc_dir.rglob("*")):
        if path.suffix not in {".html", ".js"} or not path.is_file():
            continue
        text = path.read_text(encoding="utf-8", errors="replace").lower()
        for token in FORBIDDEN_PUBLIC_TOKENS:
            if token in text:
                leaks.append(f"{path.relative_to(doc_dir)}: {token}")
    if leaks:
        raise ValueError("hidden implementation crate leaked into rustdoc:\n  " + "\n  ".join(leaks))


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--profile",
        choices=("wpa2-personal", "wpa3-personal"),
        required=True,
    )
    parser.add_argument("--rustdoc", type=Path)
    args = parser.parse_args()

    check_source(args.profile)
    check_consumer(args.profile)
    if args.rustdoc is not None:
        check_rustdoc(args.rustdoc)
    print(f"RF facade boundaries OK for {args.profile}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
