"""Regression checks for portable archive and license validation (stdlib only)."""
import hashlib
import json
from pathlib import Path
import shutil
import tempfile
import unittest
import zipfile

import package_windows as package


class PackageValidation(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        target = package.ROOT / "target"
        target.mkdir(exist_ok=True)
        if not target.resolve().is_relative_to(package.ROOT.resolve()):
            raise RuntimeError("Test target escaped workspace")
        # Preserve fresh owned scratch; no recursive cleanup of computed paths.
        cls.scratch = Path(tempfile.mkdtemp(prefix="task23-regression-", dir=target))

    def archive(self, payloads, *, tamper=False, duplicate=False, raw_names=None):
        root = "Authenticator-0.1.0-windows-x86_64"
        payloads = {"Authenticator.exe":b"public", "README.md":b"public", "User Guide.md":b"public", "Security Limits.md":b"public", "LICENSE-MIT":b"public", "LICENSE-APACHE":b"public", "NOTICE.txt":b"public", "THIRD-PARTY-NOTICES.json":b"[]", **payloads}
        record = {"application": package.APP, "target": package.TARGET, "version":"0.1.0", "files": []}
        for name, data in payloads.items():
            record["files"].append({"path": name, "sha256": hashlib.sha256(data).hexdigest(), "bytes":len(data)})
        if duplicate:
            record["files"].append(record["files"][0])
        path = self.scratch / (self.id().split(".")[-1]+".zip")
        with zipfile.ZipFile(path, "w") as archive:
            for name, data in payloads.items():
                archive.writestr(root+"/"+(raw_names or {}).get(name,name), data+b"changed" if tamper else data)
            archive.writestr(root+"/build-info.json", json.dumps(record))
        return path

    def test_valid_manifest(self):
        package.validate(self.archive({"README.md":b"public"}))

    def test_manifest_tamper_rejected(self):
        with self.assertRaisesRegex(RuntimeError, "checksum"):
            package.validate(self.archive({"README.md":b"public"},tamper=True))

    def test_duplicate_manifest_rejected(self):
        with self.assertRaisesRegex(RuntimeError, "manifest"):
            package.validate(self.archive({"README.md":b"public"},duplicate=True))

    def test_unsafe_path_rejected(self):
        with self.assertRaisesRegex(RuntimeError, "Unsafe"):
            package.validate(self.archive({"../README.md":b"public"}))

    def test_userdata_rejected_even_when_in_manifest(self):
        with self.assertRaisesRegex(RuntimeError, "user-data"):
            package.validate(self.archive({"licenses/disposable.authpbk":b"secret"}))

    def test_windows_path_alias_rejected(self):
        with self.assertRaisesRegex(RuntimeError, "Windows ZIP path"):
            package.validate(self.archive({"licenses/LICENSE.":b"public"}))

    def test_windows_device_aliases_rejected(self):
        for alias in ("COM¹.txt", "com²", "COM³.log", "LPT¹", "LPT².txt", "lpt³", "CONIN$", "conout$.txt",
                      "CON .txt", "COM1 .txt", "CONIN$ .txt", "com¹  .txt", "lpt² .txt", "conout$   .log"):
            with self.subTest(alias=alias), self.assertRaisesRegex(RuntimeError, "Windows ZIP path"):
                package.validate(self.archive({"licenses/"+alias:b"public"}))

    def test_file_ancestor_collisions_rejected_in_both_orders(self):
        for names in (("licenses/collision", "licenses/collision/child.txt"),
                      ("licenses/COLLISION/child.txt", "licenses/collision")):
            with self.subTest(names=names), self.assertRaisesRegex(RuntimeError, "ancestor collision"):
                package.validate(self.archive(dict.fromkeys(names,b"public")))

    def test_raw_nul_name_rejected_before_crc_or_manifest(self):
        path = self.archive({"licenses/LICENSE":b"public"},
                            raw_names={"licenses/LICENSE":"licenses/LICENSE@hidden"})
        raw = path.read_bytes()
        self.assertEqual(raw.count(b"licenses/LICENSE@hidden"),2)
        path.write_bytes(raw.replace(b"licenses/LICENSE@hidden",b"licenses/LICENSE\x00hidden"))
        with zipfile.ZipFile(path) as archive:
            entry = next(e for e in archive.infolist() if "hidden" in e.orig_filename)
            self.assertNotEqual(entry.orig_filename,entry.filename)
        with self.assertRaisesRegex(RuntimeError, "Unsafe raw ZIP path"):
            package.validate(path)

    def test_non_allowlisted_payload_rejected(self):
        with self.assertRaisesRegex(RuntimeError, "allowlist"):
            package.validate(self.archive({"unexpected.txt":b"public"}))

    def test_license_and_requires_all_clauses(self):
        mit = package.ROOT / "LICENSE-MIT"
        self.assertEqual(package.license_choice("MIT OR Apache-2.0",[mit]),["MIT"])
        self.assertIsNone(package.license_choice("MIT AND Apache-2.0",[mit]))
        self.assertIsNone(package.license_choice("(MIT OR Apache-2.0) AND OFL-1.1",[mit]))
        self.assertEqual(package.license_choice("MIT AND Apache-2.0",[mit,package.ROOT/"LICENSE-APACHE"]),["Apache-2.0","MIT"])

    def test_notice_only_cannot_satisfy_license(self):
        notice = self.scratch / "NOTICE"
        notice.write_text("Attribution only. Copyright contributors.")
        self.assertIsNone(package.license_choice("MIT",[notice]))

    def test_discrepancy_requires_exact_version_and_hash(self):
        notice = self.scratch / "LICENSE"
        notice.write_text("Unreviewed different license")
        with self.assertRaisesRegex(RuntimeError,"discrepancy changed"):
            package.packaged_license({"name":"quircs","version":"0.10.3","license":"MIT"},[notice])

    def test_vendored_notices_survive_windows_autocrlf_checkout(self):
        # Exercise real Git checkout filters; source SHA verification must survive
        # core.autocrlf=true without changing a recorded upstream digest.
        repo = self.scratch / "notice-checkout"
        repo.mkdir()
        shutil.copytree(package.ROOT / "assets/notices", repo / "assets/notices")
        shutil.copyfile(package.ROOT / ".gitattributes", repo / ".gitattributes")
        package.command("git", "init", "--quiet", str(repo))
        package.command("git", "-C", str(repo), "-c", "core.autocrlf=true", "add", "--", ".gitattributes", "assets/notices")
        exported = repo / "exported"
        exported.mkdir()
        package.command("git", "-C", str(repo), "-c", "core.autocrlf=true", "checkout-index", "--all", "--prefix="+exported.as_posix()+"/")
        checked = 0
        for provenance in (exported / "assets/notices").glob("*/provenance.json"):
            for item in json.loads(provenance.read_text(encoding="utf-8"))["files"]:
                self.assertEqual(package.sha(provenance.parent / item["file"]),item["sha256"])
                checked += 1
        self.assertGreater(checked,0)
        # The same public LF text without an attribute really is converted.
        control = self.scratch / "notice-control"
        control.mkdir()
        public = package.ROOT / "assets/notices/accesskit-0.25.2/LICENSE-MIT"
        shutil.copyfile(public,control / "LICENSE")
        package.command("git", "init", "--quiet", str(control))
        package.command("git", "-C", str(control), "-c", "core.autocrlf=true", "add", "--", "LICENSE")
        destination = control / "exported"
        destination.mkdir()
        package.command("git", "-C", str(control), "-c", "core.autocrlf=true", "checkout-index", "--all", "--prefix="+destination.as_posix()+"/")
        self.assertNotEqual(package.sha(public),package.sha(destination / "LICENSE"))
        self.assertIn(b"\r\n",(destination / "LICENSE").read_bytes())


if __name__ == "__main__":
    unittest.main()
