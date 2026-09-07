#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
"""Build an isolated consumer against a candidate or published hisi-rf package."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import shutil
import subprocess
import tarfile
import tempfile
import time
import tomllib
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
FIXTURE = ROOT / ".github" / "fixtures" / "ws63-consumer"
TARGET = "riscv32imfc-unknown-none-elf"
PROFILES = ("wpa2-personal", "wpa3-personal", "ble-peripheral", "sle-announce")
RADIO_PROFILES = {"ble-peripheral", "sle-announce"}
DEPENDENCY_LINE = re.compile(r"^hisi-rf\s*=.*$", re.MULTILINE)


def run(
    command: list[str],
    *,
    cwd: Path,
    capture: bool = False,
    check: bool = True,
) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        command,
        cwd=cwd,
        check=check,
        capture_output=capture,
        text=True,
    )


def copy_fixture(destination: Path) -> None:
    shutil.copytree(
        FIXTURE,
        destination,
        ignore=shutil.ignore_patterns("Cargo.lock", "target", "__pycache__"),
    )


def replace_dependency(manifest: Path, replacement: str) -> None:
    original = manifest.read_text(encoding="utf-8")
    updated, count = DEPENDENCY_LINE.subn(replacement, original)
    if count != 1:
        raise RuntimeError(f"expected one hisi-rf dependency in {manifest}, found {count}")
    manifest.write_text(updated, encoding="utf-8")


def safe_extract(crate: Path, destination: Path) -> Path:
    destination_root = destination.resolve()
    with tarfile.open(crate, mode="r:gz") as archive:
        members = archive.getmembers()
        for member in members:
            member_path = (destination / member.name).resolve()
            if destination_root not in member_path.parents:
                raise RuntimeError(f"unsafe path in candidate archive: {member.name}")
            if member.issym() or member.islnk():
                raise RuntimeError(f"links are not accepted in candidate archive: {member.name}")
        archive.extractall(destination, members=members, filter="data")

    manifests = sorted(destination.glob("hisi-rf-*/Cargo.toml"))
    if len(manifests) != 1:
        raise RuntimeError(
            f"expected one packaged hisi-rf manifest, found {len(manifests)}"
        )
    return manifests[0].parent


def package_version(package_root: Path) -> str:
    with (package_root / "Cargo.toml").open("rb") as source:
        return tomllib.load(source)["package"]["version"]


def generate_lock(consumer: Path, retries: int, retry_seconds: int) -> None:
    for attempt in range(1, retries + 1):
        completed = run(
            ["cargo", "generate-lockfile"],
            cwd=consumer,
            capture=True,
            check=False,
        )
        if completed.returncode == 0:
            return
        if attempt == retries:
            raise RuntimeError(
                "cargo generate-lockfile failed after registry propagation retries:\n"
                + completed.stderr.strip()
            )
        time.sleep(retry_seconds)


def metadata(consumer: Path, profile: str) -> dict:
    completed = run(
        [
            "cargo",
            "metadata",
            "--locked",
            "--format-version",
            "1",
            "--filter-platform",
            TARGET,
            "--features",
            profile,
        ],
        cwd=consumer,
        capture=True,
    )
    return json.loads(completed.stdout)


def verify_graph(meta: dict, mode: str, expected_version: str, package_root: Path | None) -> None:
    packages = {package["name"]: package for package in meta["packages"]}
    facade = packages.get("hisi-rf")
    if facade is None or facade["version"] != expected_version:
        actual = None if facade is None else facade["version"]
        raise RuntimeError(f"resolved hisi-rf {actual}, expected {expected_version}")

    source = facade.get("source") or ""
    manifest = Path(facade["manifest_path"]).resolve()
    if mode == "candidate":
        if source or package_root is None or manifest.parent != package_root.resolve():
            raise RuntimeError("candidate consumer did not resolve the extracted package")
    elif not source.startswith("registry+"):
        raise RuntimeError(f"published consumer resolved hisi-rf outside a registry: {source}")

    for name in ("hisi-rf-core", "hisi-rf-ws63", "ws63-radio-sys", "ws63-radio-blob"):
        package = packages.get(name)
        if package is None:
            raise RuntimeError(f"consumer graph is missing {name}")
        dependency_source = package.get("source") or ""
        if not dependency_source.startswith("registry+"):
            raise RuntimeError(f"{name} resolved outside a registry: {dependency_source}")


def build(consumer: Path, profile: str) -> None:
    command = [
        "cargo",
        "build",
        "--quiet",
        "--release",
        "--locked",
        "--features",
        profile,
    ]
    if profile in RADIO_PROFILES:
        command.extend(["--bin", "radio"])
    run(command, cwd=consumer)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("mode", choices=("candidate", "published"))
    parser.add_argument("--profile", choices=PROFILES, required=True)
    parser.add_argument("--crate", type=Path)
    parser.add_argument("--version")
    parser.add_argument("--report", type=Path)
    parser.add_argument("--registry-retries", type=int, default=12)
    parser.add_argument("--retry-seconds", type=int, default=10)
    args = parser.parse_args()

    if args.mode == "candidate" and (args.crate is None or args.version is not None):
        parser.error("candidate mode requires --crate and does not accept --version")
    if args.mode == "published" and (args.version is None or args.crate is not None):
        parser.error("published mode requires --version and does not accept --crate")

    temp_root = Path(os.environ.get("RUNNER_TEMP", tempfile.gettempdir()))
    with tempfile.TemporaryDirectory(
        dir=temp_root,
        prefix="hisi rf release evidence ",
    ) as temp_dir:
        workspace = Path(temp_dir)
        package_root = None
        if args.mode == "candidate":
            candidate = args.crate.resolve()
            if candidate.is_dir():
                candidates = sorted(candidate.glob("hisi-rf-*.crate"))
                if len(candidates) != 1:
                    raise RuntimeError(
                        f"expected one hisi-rf candidate in {candidate}, "
                        f"found {len(candidates)}"
                    )
                candidate = candidates[0]
            if not candidate.is_file():
                raise RuntimeError(f"candidate crate does not exist: {candidate}")
            package_root = safe_extract(candidate, workspace / "package")
            version = package_version(package_root)
            dependency = (
                "hisi-rf = { path = "
                + json.dumps(str(package_root))
                + ', features = ["chip-ws63"] }'
            )
            archive_sha256 = hashlib.sha256(candidate.read_bytes()).hexdigest()
            retries = 1
        else:
            version = args.version
            dependency = (
                f'hisi-rf = {{ version = "={version}", features = ["chip-ws63"] }}'
            )
            archive_sha256 = None
            retries = args.registry_retries

        consumer = workspace / "consumer"
        copy_fixture(consumer)
        replace_dependency(consumer / "Cargo.toml", dependency)
        generate_lock(consumer, retries, args.retry_seconds)
        meta = metadata(consumer, args.profile)
        verify_graph(meta, args.mode, version, package_root)
        build(consumer, args.profile)

        report = {
            "schema": 1,
            "mode": args.mode,
            "profile": args.profile,
            "facade_version": version,
            "candidate_sha256": archive_sha256,
            "target": TARGET,
            "status": "pass",
        }
        rendered = json.dumps(report, indent=2, sort_keys=True)
        if args.report is not None:
            args.report.parent.mkdir(parents=True, exist_ok=True)
            args.report.write_text(rendered + "\n", encoding="utf-8")
        print(rendered)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
