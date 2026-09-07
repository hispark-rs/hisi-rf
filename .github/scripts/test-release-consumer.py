#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
"""Cross-host path serialization regressions for packaged consumer evidence."""
import importlib.util
import json
from pathlib import Path
import sys
import tempfile
import tomllib
import unittest

sys.dont_write_bytecode = True
spec = importlib.util.spec_from_file_location("consumer", Path(__file__).with_name("check-release-consumer.py"))
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


class ConsumerPaths(unittest.TestCase):
    def test_toml_path_roundtrip(self):
        for path in (r"D:\a\_temp\hisi rf\package", r"C:\Users\构建\release", "/tmp/space and unicode 构建/package"):
            with self.subTest(path=path), tempfile.TemporaryDirectory() as directory:
                manifest = Path(directory) / "Cargo.toml"
                manifest.write_text('[dependencies]\nhisi-rf = "=0.1.0"\n', encoding="utf-8")
                module.replace_dependency(manifest, 'hisi-rf = { path = ' + json.dumps(path) + ' }')
                self.assertEqual(tomllib.loads(manifest.read_text(encoding="utf-8"))["dependencies"]["hisi-rf"]["path"], path)


if __name__ == "__main__":
    unittest.main()
