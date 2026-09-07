#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = ["pyyaml==6.0.2"]
# ///
"""Negative and unit tests for the release consumer evidence gate."""

from __future__ import annotations

import importlib.util
import io
import json
import sys
import tomllib
import tarfile
import tempfile
import unittest
from pathlib import Path
import yaml


sys.dont_write_bytecode = True
SCRIPT = Path(__file__).with_name("check-release-consumer.py")
SPEC = importlib.util.spec_from_file_location("check_release_consumer", SCRIPT)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError(f"cannot load {SCRIPT}")
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


def package(name: str, version: str, source: str | None, manifest: Path) -> dict:
    return {
        "name": name,
        "version": version,
        "source": source,
        "manifest_path": str(manifest),
    }


class ReleaseConsumerTests(unittest.TestCase):
    def test_downloaded_candidates_do_not_dirty_the_release_checkout(self):
        workflow = yaml.safe_load((SCRIPT.parents[1] / "workflows/publish.yml").read_text())
        for job_name in ("candidate-consumer", "publish"):
            steps = workflow["jobs"][job_name]["steps"]
            downloads = [step for step in steps if step.get("uses", "").startswith("actions/download-artifact@")]
            self.assertEqual(len(downloads), 1)
            self.assertEqual(downloads[0]["with"]["path"], "${{ runner.temp }}/candidate")
        publish_steps = workflow["jobs"]["publish"]["steps"]
        repackage = next(step for step in publish_steps if step.get("name") == "Reproduce candidate package")
        self.assertIn("${RUNNER_TEMP}/candidate/", repackage["run"])
        self.assertNotIn("--allow-dirty", repackage["run"])

    def test_toml_path_roundtrip(self):
        for path in (r"D:\a\_temp\hisi rf\package", r"C:\Users\构建\release", "/tmp/space and unicode 构建/package"):
            with self.subTest(path=path), tempfile.TemporaryDirectory() as directory:
                manifest = Path(directory) / "Cargo.toml"
                manifest.write_text('[dependencies]\nhisi-rf = "=0.1.0"\n', encoding="utf-8")
                MODULE.replace_dependency(manifest, 'hisi-rf = { path = ' + json.dumps(path) + ' }')
                self.assertEqual(tomllib.loads(manifest.read_text(encoding="utf-8"))["dependencies"]["hisi-rf"]["path"], path)

    def test_replace_dependency_requires_exactly_one_entry(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            manifest = Path(temp_dir) / "Cargo.toml"
            manifest.write_text("[dependencies]\nfoo = \"1\"\n", encoding="utf-8")
            with self.assertRaisesRegex(RuntimeError, "found 0"):
                MODULE.replace_dependency(manifest, 'hisi-rf = "=1.0.0"')

            manifest.write_text(
                '[dependencies]\nhisi-rf = "=1"\nhisi-rf = "=2"\n',
                encoding="utf-8",
            )
            with self.assertRaisesRegex(RuntimeError, "found 2"):
                MODULE.replace_dependency(manifest, 'hisi-rf = "=1.0.0"')

    def test_safe_extract_rejects_path_traversal(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            archive_path = root / "candidate.crate"
            with tarfile.open(archive_path, "w:gz") as archive:
                payload = b"escape"
                member = tarfile.TarInfo("../outside")
                member.size = len(payload)
                archive.addfile(member, io.BytesIO(payload))

            with self.assertRaisesRegex(RuntimeError, "unsafe path"):
                MODULE.safe_extract(archive_path, root / "extract")

    def test_safe_extract_rejects_links(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            archive_path = root / "candidate.crate"
            with tarfile.open(archive_path, "w:gz") as archive:
                member = tarfile.TarInfo("hisi-rf-1.0.0/link")
                member.type = tarfile.SYMTYPE
                member.linkname = "Cargo.toml"
                archive.addfile(member)

            with self.assertRaisesRegex(RuntimeError, "links are not accepted"):
                MODULE.safe_extract(archive_path, root / "extract")

    def test_candidate_graph_requires_extracted_facade_and_registry_internals(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            package_root = Path(temp_dir) / "hisi-rf-1.0.0"
            registry = "registry+https://github.com/rust-lang/crates.io-index"
            packages = [
                package("hisi-rf", "1.0.0", None, package_root / "Cargo.toml"),
                *[
                    package(name, "1.0.0", registry, Path("registry") / name / "Cargo.toml")
                    for name in (
                        "hisi-rf-core",
                        "hisi-rf-ws63",
                        "ws63-radio-sys",
                        "ws63-radio-blob",
                    )
                ],
            ]
            MODULE.verify_graph({"packages": packages}, "candidate", "1.0.0", package_root)

            packages[-1]["source"] = None
            with self.assertRaisesRegex(RuntimeError, "resolved outside a registry"):
                MODULE.verify_graph(
                    {"packages": packages}, "candidate", "1.0.0", package_root
                )

    def test_published_graph_rejects_path_facade(self) -> None:
        registry = "registry+https://github.com/rust-lang/crates.io-index"
        packages = [
            package("hisi-rf", "1.0.0", None, Path("source") / "Cargo.toml"),
            *[
                package(name, "1.0.0", registry, Path("registry") / name / "Cargo.toml")
                for name in (
                    "hisi-rf-core",
                    "hisi-rf-ws63",
                    "ws63-radio-sys",
                    "ws63-radio-blob",
                )
            ],
        ]
        with self.assertRaisesRegex(RuntimeError, "outside a registry"):
            MODULE.verify_graph({"packages": packages}, "published", "1.0.0", None)


if __name__ == "__main__":
    unittest.main()
