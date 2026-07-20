#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
"""Exercise registry immutability and concurrent external RF builds."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import stat
import subprocess
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
CONSUMER = ROOT / ".github" / "fixtures" / "ws63-consumer"
TARGET = "riscv32imfc-unknown-none-elf"


def run(command: list[str], *, env: dict[str, str] | None = None) -> None:
    subprocess.run(command, cwd=CONSUMER, env=env, check=True)


def consumer_command(profile: str, *, offline: bool = True) -> list[str]:
    command = [
        "cargo",
        "build",
        "--release",
        "--locked",
        "--features",
        f"hisi-rf/{profile}",
    ]
    if offline:
        command.insert(4, "--offline")
    return command


def registry_package_roots(profile: str) -> list[Path]:
    completed = subprocess.run(
        [
            "cargo",
            "metadata",
            "--locked",
            "--offline",
            "--format-version",
            "1",
            "--filter-platform",
            TARGET,
            "--features",
            f"hisi-rf/{profile}",
        ],
        cwd=CONSUMER,
        check=True,
        capture_output=True,
        text=True,
    )
    metadata = json.loads(completed.stdout)
    roots = {
        Path(package["manifest_path"]).resolve().parent
        for package in metadata["packages"]
        if (package.get("source") or "").startswith("registry+")
    }
    if not roots:
        raise RuntimeError("consumer metadata did not contain registry packages")
    return sorted(roots)


def files_under(roots: list[Path]) -> list[Path]:
    return sorted(
        path
        for root in roots
        for path in root.rglob("*")
        if path.is_file() and not path.is_symlink()
    )


def digest_files(files: list[Path]) -> dict[Path, str]:
    return {
        path: hashlib.sha256(path.read_bytes()).hexdigest()
        for path in files
    }


def readonly_build(profile: str) -> None:
    roots = registry_package_roots(profile)
    files = files_under(roots)
    before = digest_files(files)
    modes = {
        path: stat.S_IMODE(path.stat().st_mode)
        for root in roots
        for path in [root, *root.rglob("*")]
        if not path.is_symlink()
    }

    try:
        for path, mode in sorted(modes.items(), key=lambda item: len(item[0].parts), reverse=True):
            path.chmod(mode & ~(stat.S_IWUSR | stat.S_IWGRP | stat.S_IWOTH))
        run(["cargo", "clean"])
        run(consumer_command(profile))
        after_files = files_under(roots)
        if after_files != files:
            added = sorted(str(path) for path in set(after_files) - set(files))
            removed = sorted(str(path) for path in set(files) - set(after_files))
            raise RuntimeError(
                "registry source file set changed during build: "
                f"added={added}, removed={removed}"
            )
        after = digest_files(after_files)
        if after != before:
            changed = sorted(str(path) for path in before if before[path] != after.get(path))
            raise RuntimeError("registry source changed during build: " + ", ".join(changed))
    finally:
        for path, mode in sorted(modes.items(), key=lambda item: len(item[0].parts)):
            if path.exists():
                path.chmod(mode)

    print(f"read-only registry build OK for {profile} ({len(roots)} packages)")


def concurrent_builds() -> None:
    run(["cargo", "fetch", "--locked", "--target", TARGET])
    temp_root = Path(os.environ.get("RUNNER_TEMP", tempfile.gettempdir()))
    target_root = temp_root / "hisi rf concurrent 构建"
    processes: list[tuple[str, subprocess.Popen[str]]] = []
    for profile in ("wpa2-personal", "wpa3-personal"):
        env = os.environ.copy()
        env["CARGO_TARGET_DIR"] = str(target_root / profile)
        process = subprocess.Popen(
            consumer_command(profile, offline=False),
            cwd=CONSUMER,
            env=env,
            text=True,
        )
        processes.append((profile, process))

    failures = [profile for profile, process in processes if process.wait() != 0]
    if failures:
        raise RuntimeError("concurrent consumer build failed: " + ", ".join(failures))
    print("concurrent WPA2/WPA3 consumer builds OK")


def main() -> int:
    parser = argparse.ArgumentParser()
    subparsers = parser.add_subparsers(dest="command", required=True)
    readonly = subparsers.add_parser("readonly")
    readonly.add_argument(
        "--profile",
        choices=("wpa2-personal", "wpa3-personal"),
        required=True,
    )
    subparsers.add_parser("concurrent")
    args = parser.parse_args()

    if args.command == "readonly":
        readonly_build(args.profile)
    else:
        concurrent_builds()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
