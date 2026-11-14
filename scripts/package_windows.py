"""Build and validate an allowlisted Windows x64 portable package. Python 3.11+."""
from __future__ import annotations
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import struct
import subprocess
import sys
import tempfile
from datetime import datetime, timezone
import zipfile
import zlib

ROOT = Path(__file__).resolve().parents[1]
TARGET = "x86_64-pc-windows-msvc"
APP = "Authenticator"
RELEASE_RUSTFLAGS = "-C target-feature=+crt-static -C link-arg=/Brepro"


def source_epoch() -> int:
    """Use the source timestamp, never the build's wall clock."""
    value = os.environ.get("SOURCE_DATE_EPOCH")
    if value is None:
        value = command("git", "log", "-1", "--format=%ct")
    if not re.fullmatch(r"[0-9]{1,12}", value):
        raise RuntimeError("SOURCE_DATE_EPOCH must be an unsigned integer timestamp")
    epoch = int(value)
    if epoch > 4354819199:  # ZIP's last representable UTC year is 2107.
        raise RuntimeError("SOURCE_DATE_EPOCH exceeds the supported ZIP timestamp range")
    return epoch


def write_json(path: Path, value: object) -> None:
    path.write_text(json.dumps(value, indent=2)+"\n", encoding="utf-8", newline="\n")


def write_archive(stage: Path, name: str, path: Path, epoch: int) -> None:
    # ZIP stores UTC-derived DOS timestamps at two-second precision. Fix all
    # platform attributes instead of inheriting copy-time mtimes/host permissions.
    stamp = datetime.fromtimestamp(max(epoch, 315532800), timezone.utc)
    date = (stamp.year, stamp.month, stamp.day, stamp.hour, stamp.minute, stamp.second // 2 * 2)
    with zipfile.ZipFile(path, "x", compression=zipfile.ZIP_DEFLATED, compresslevel=9) as archive:
        for source in sorted((p for p in stage.rglob("*") if p.is_file()), key=lambda p: p.relative_to(stage).as_posix()):
            entry = zipfile.ZipInfo(f"{name}/{source.relative_to(stage).as_posix()}", date)
            entry.create_system = 3
            entry.external_attr = 0o100644 << 16
            entry.compress_type = zipfile.ZIP_DEFLATED
            archive.writestr(entry, source.read_bytes(), compresslevel=9)

def command(*args: str) -> str:
    result = subprocess.run(args, cwd=ROOT, text=True, encoding="utf-8", capture_output=True)
    if result.returncode:
        # Build diagnostics contain source/tool paths, never runtime vault input.
        raise RuntimeError(f"Command failed: {args[0]} {args[1:]}\n{result.stderr[-12000:]}")
    return result.stdout.strip()

def sha(path: Path) -> str:
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()

def metadata() -> tuple[dict, list[dict]]:
    meta = json.loads(command("cargo", "metadata", "--locked", "--format-version", "1", "--filter-platform", TARGET))
    tree = command("cargo", "tree", "--locked", "--target", TARGET, "--edges", "normal,build", "--prefix", "none", "--format", "{p}")
    keys = {tuple(m.groups()) for line in tree.splitlines() if (m := re.match(r"^(\S+) v(\S+)", line))}
    packages = [p for p in meta["packages"] if (p["name"], p["version"]) in keys]
    if len(packages) != len(keys):
        raise RuntimeError("Dependency graph has ambiguous or missing package identities")
    own = next(p for p in packages if Path(p["manifest_path"]).resolve() == ROOT / "Cargo.toml")
    return own, sorted(packages, key=lambda p: (p["name"], p["version"]))

def license_files(package: dict) -> list[Path]:
    folder = Path(package["manifest_path"]).parent
    files = []
    for path in folder.rglob("*"):
        if not path.is_file() or path.is_symlink():
            continue
        name = path.name.upper()
        if re.match(r"^(LICEN[CS]E|COPYING|NOTICE|COPYRIGHT|UNLICENSE)([._-]|$)", name):
            files.append(path)
        elif package["name"] == "epaint_default_fonts" and path.suffix == ".txt" and path.parent.name == "fonts":
            files.append(path)
    if package.get("license_file"):
        explicit = Path(package["license_file"])
        if not explicit.is_absolute():
            explicit = folder / explicit
        if not explicit.resolve().is_relative_to(folder.resolve()):
            raise RuntimeError("License path escaped package source")
        if explicit not in files and explicit.is_file():
            files.append(explicit)
    vendored = ROOT / "assets/notices" / f"{package['name']}-{package['version']}"
    if (vendored / "provenance.json").is_file() and not vendored.is_symlink():
        provenance = json.loads((vendored / "provenance.json").read_text(encoding="utf-8"))
        for item in provenance["files"]:
            notice = vendored / item["file"]
            if not notice.resolve().is_relative_to(vendored.resolve()) or sha(notice) != item["sha256"]:
                raise RuntimeError(f"Vendored upstream notice failed verification: {package['name']}-{package['version']}/{item['file']}")
            files.append(notice)
    # Absolute registry/vendored root names differ between machines. The same
    # relative source identities must determine notice numbering everywhere.
    def identity(path: Path) -> str:
        return "crate/"+path.relative_to(folder).as_posix() if path.is_relative_to(folder) else "vendored/"+path.name
    return sorted(set(files), key=identity)

def license_choice(expression: str, files: list[Path]) -> list[str] | None:
    """Check every AND clause/one OR choice against full known text families.

    This supports this lockfile's declarations, not a general legal/SPDX engine.
    Unknown expressions or unavailable text fail closed.
    """
    text = " ".join(" ".join(p.read_text(encoding="utf-8",errors="replace").lower().split()) for p in files)
    available = set()
    if "apache license" in text and "version 2.0" in text and "terms and conditions for use, reproduction, and distribution" in text:
        available.add("Apache-2.0")
    if "permission is hereby granted, free of charge" in text and "the software is provided" in text:
        if "the above copyright notice and this permission notice shall be included" in text:
            available.add("MIT")
        else:
            available.add("MIT-0")
    if "sil open font license" in text and "version 1.1" in text and "definitions" in text and "termination" in text:
        available.add("OFL-1.1")
    if "ubuntu font licence" in text and "version 1.0" in text and "preamble" in text:
        available.add("Ubuntu-font-1.0")
    if "unicode license v3" in text and "copyright and permission notice" in text:
        available.add("Unicode-3.0")
    if "redistribution and use in source and binary forms" in text:
        available.add("BSD-3-Clause" if "neither the name" in text else "BSD-2-Clause")
    if "permission to use, copy, modify, and/or distribute this software" in text and "with or without fee" in text:
        available.add("ISC")
    if "origin of this software must not be misrepresented" in text and "altered source versions" in text:
        available.add("Zlib")
    if "free and unencumbered software released into the public domain" in text:
        available.add("Unlicense")
    if "cc0 1.0 universal" in text and "statement of purpose" in text:
        available.add("CC0-1.0")
    if "permission to use, copy, modify, and/or distribute this software" in text and "copyright notice" not in text:
        available.add("0BSD")
    normalized = expression.replace("MIT/Apache-2.0", "MIT OR Apache-2.0")
    tokens = re.findall(r"\(|\)|[A-Za-z0-9.-]+", normalized)
    if "".join(tokens) != re.sub(r"\s+", "", normalized) or len(tokens) > 64:
        raise RuntimeError("Unsupported license expression syntax")
    index = 0
    def atom():
        nonlocal index
        if index >= len(tokens):
            raise RuntimeError("Incomplete license expression")
        token = tokens[index]
        index += 1
        if token == "(":
            value = disjunction()
            if index >= len(tokens) or tokens[index] != ")":
                raise RuntimeError("Unbalanced license expression")
            index += 1
            return value
        return {token} if token in available else None
    def conjunction():
        nonlocal index
        left = atom()
        while index < len(tokens) and tokens[index] == "AND":
            index += 1
            right = atom()
            left = left | right if left is not None and right is not None else None
        return left
    def disjunction():
        nonlocal index
        left = conjunction()
        while index < len(tokens) and tokens[index] == "OR":
            index += 1
            right = conjunction()
            left = left if left is not None else right
        return left
    choice = disjunction()
    if index != len(tokens):
        raise RuntimeError("Unsupported license expression")
    return sorted(choice) if choice is not None else None

def packaged_license(package: dict, files: list[Path]) -> tuple[str, str | None]:
    """Retain metadata discrepancies rather than inventing a missing grant."""
    if (package["name"], package["version"]) == ("quircs", "0.10.3"):
        expected = "166760d656f6bfc29dfc76a7f763e529bde0ed22dde2b20cf6b6c4b14a138a9a"
        if package.get("license") != "MIT" or not any(p.name == "LICENSE" and sha(p) == expected for p in files):
            raise RuntimeError("Reviewed quircs license discrepancy changed")
        return "ISC", ("Published Cargo metadata says MIT, but this version's bundled LICENSE "
                       "contains the ISC grant. The exact published text is retained, SHA256 " + expected +
                       ". See https://docs.rs/crate/quircs/0.10.3/source/LICENSE and https://spdx.org/licenses/ISC.html.")
    return package.get("license") or "", None


def notices(packages: list[dict], destination: Path) -> list[dict]:
    destination.mkdir(parents=True)
    inventory = []
    for package in packages:
        if package["name"] == "authenticator" and package["source"] is None:
            continue
        if not package.get("license") and not package.get("license_file"):
            raise RuntimeError(f"Missing license declaration: {package['name']}")
        folder = Path(package["manifest_path"]).parent
        files = license_files(package)
        if not files:
            raise RuntimeError(f"No distributed license text for {package['name']} {package['version']}")
        expression, discrepancy = packaged_license(package, files)
        choice = license_choice(expression, files)
        if choice is None:
            raise RuntimeError(f"Incomplete declared license-expression texts: {package['name']} {package['license']}")
        prefix = f"{package['name']}-{package['version']}"
        if not re.fullmatch(r"[A-Za-z0-9_.+-]+", prefix):
            raise RuntimeError("Unsafe dependency identity")
        copied = []
        for index, path in enumerate(files):
            if path.stat().st_size > 2 * 1024 * 1024:
                raise RuntimeError(f"Unreasonably large notice file: {prefix}")
            relative = path.relative_to(folder).as_posix() if path.is_relative_to(folder) else path.name
            name = f"{index:02d}-{path.name}"
            out = destination / prefix / name
            out.parent.mkdir(exist_ok=True)
            shutil.copyfile(path, out)
            copied.append({"source": relative, "file": f"licenses/{prefix}/{name}", "sha256": sha(out)})
        record = {"name": package["name"], "version": package["version"], "license": package.get("license"), "selected_text_families":choice,"repository": package.get("repository"), "authors":package.get("authors",[]),"notices": copied}
        record["packaged_license_expression"] = expression
        if discrepancy:
            record["license_discrepancy"] = discrepancy
        provenance = ROOT / "assets/notices" / prefix / "provenance.json"
        if provenance.is_file():
            record["upstream_notice_provenance"] = json.loads(provenance.read_text(encoding="utf-8"))
        inventory.append(record)
    noto = destination / "NotoSans"
    noto.mkdir()
    shutil.copyfile(ROOT / "assets/fonts/OFL.txt", noto / "OFL.txt")
    inventory.append({"name": "Noto Sans bundled Regular and SemiBold", "license": "OFL-1.1", "notices": [{"file": "licenses/NotoSans/OFL.txt", "sha256": sha(noto / "OFL.txt")} ]})
    sysroot = Path(command("rustc", "--print", "sysroot"))
    runtime_notice = sysroot / "share/doc/rust/COPYRIGHT-library.html"
    if not runtime_notice.is_file():
        raise RuntimeError("Installed Rust standard-library copyright notice is required")
    runtime = destination / "Rust-standard-library"
    runtime.mkdir()
    shutil.copyfile(runtime_notice, runtime / runtime_notice.name)
    inventory.append({"name":"Rust standard library and bundled runtime components","toolchain":command("rustc","--version"),"notices":[{"file":"licenses/Rust-standard-library/COPYRIGHT-library.html","sha256":sha(runtime/runtime_notice.name)}]})
    return inventory

def pe_info(path: Path) -> dict:
    data = path.read_bytes()
    if len(data) > 128 * 1024 * 1024 or data[:2] != b"MZ":
        raise RuntimeError("Not a bounded Windows PE file")
    pe = struct.unpack_from("<I", data, 0x3C)[0]
    if data[pe:pe+4] != b"PE\0\0":
        raise RuntimeError("Missing PE header")
    machine, sections = struct.unpack_from("<HH", data, pe+4)
    optional_size = struct.unpack_from("<H", data, pe+20)[0]
    optional = pe+24
    if machine != 0x8664 or struct.unpack_from("<H", data, optional)[0] != 0x20B:
        raise RuntimeError("Package must contain x64 PE32+")
    if struct.unpack_from("<H", data, optional+68)[0] != 2:
        raise RuntimeError("Package must contain a Windows GUI executable")
    entries = []
    for index in range(sections):
        off = optional+optional_size+40*index
        virtual_size, address, size, raw = struct.unpack_from("<IIII", data, off+8)
        entries.append((address, virtual_size, raw, size))
    def offset(rva: int) -> int:
        for address, virtual_size, raw, size in entries:
            if address <= rva < address+max(virtual_size,size) and rva-address < size:
                return raw+rva-address
        raise RuntimeError("Invalid PE data directory")
    imports = []
    import_rva = struct.unpack_from("<I", data, optional+120)[0]
    if import_rva:
        base = offset(import_rva)
        for index in range(256):
            entry = struct.unpack_from("<IIIII", data, base+20*index)
            if not any(entry):
                break
            start = offset(entry[3])
            end = data.find(b"\0", start, start+256)
            if end < 0:
                raise RuntimeError("Invalid DLL import name")
            imports.append(data[start:end].decode("ascii"))
        else:
            raise RuntimeError("Unbounded import table")
    resource_rva = struct.unpack_from("<I", data, optional+128)[0]
    if not resource_rva:
        raise RuntimeError("Missing embedded Windows resources")
    resource = offset(resource_rva)
    named, ids = struct.unpack_from("<HH", data, resource+12)
    resource_types = []
    for index in range(named+ids):
        name, _ = struct.unpack_from("<II", data, resource+16+8*index)
        if not name & 0x80000000:
            resource_types.append(name)
    if not {3,14,16,24}.issubset(resource_types):
        raise RuntimeError("Icon/group icon/version/manifest resource missing")
    def payload(wanted: int) -> bytes:
        def descend(directory: int, depth: int) -> bytes:
            if depth > 3:
                raise RuntimeError("Unexpected resource tree depth")
            n, i = struct.unpack_from("<HH", data, resource+directory+12)
            if n+i > 1024:
                raise RuntimeError("Unbounded resource directory")
            for index in range(n+i):
                key, value = struct.unpack_from("<II", data, resource+directory+16+8*index)
                if depth == 0 and key != wanted:
                    continue
                if value & 0x80000000:
                    return descend(value & 0x7fffffff, depth+1)
                rva, length = struct.unpack_from("<II", data, resource+value)
                if length > 1024*1024:
                    raise RuntimeError("Unbounded resource payload")
                start = offset(rva)
                return data[start:start+length]
            raise RuntimeError("Missing resource payload")
        return descend(0,0)
    manifest = payload(24).decode("utf-8-sig").rstrip("\0").strip()
    import xml.etree.ElementTree as ET
    xml = ET.fromstring(manifest)
    execution = xml.find(".//{urn:schemas-microsoft-com:asm.v3}requestedExecutionLevel")
    dpi = xml.find(".//{http://schemas.microsoft.com/SMI/2016/WindowsSettings}dpiAwareness")
    if execution is None or execution.attrib != {"level":"asInvoker", "uiAccess":"false"} or dpi is None or dpi.text != "PerMonitorV2,PerMonitor":
        raise RuntimeError("Unexpected executable privilege/DPI manifest")
    if any(re.search(r"rust|cargo|python|node", name, re.I) for name in imports):
        raise RuntimeError("Unexpected development runtime import")
    return {"machine": "x64", "subsystem": "Windows GUI", "linker_version":f"{data[optional+2]}.{data[optional+3]}", "dll_imports": sorted(set(imports)), "resource_types": sorted(resource_types), "execution_level":"asInvoker", "dpi_awareness":dpi.text}

def build(replace: bool, skip_build: bool) -> Path:
    if os.name != "nt":
        raise RuntimeError("Windows is required to build this package")
    target_root = ROOT / "target"
    target_root.mkdir(exist_ok=True)
    if not target_root.resolve().is_relative_to(ROOT.resolve()):
        raise RuntimeError("Project target must remain inside the workspace")
    scratch = Path(tempfile.mkdtemp(prefix="task23-package-", dir=target_root)).resolve()
    if not scratch.is_relative_to(target_root.resolve()):
        raise RuntimeError("Scratch escaped project target")
    os.environ["TEMP"] = os.environ["TMP"] = str(scratch)
    os.environ["CARGO_TARGET_DIR"] = str(target_root.resolve())
    # Fixed only for this packaging process and its children; no global config.
    os.environ.pop("CARGO_ENCODED_RUSTFLAGS", None)
    os.environ["RUSTFLAGS"] = RELEASE_RUSTFLAGS
    os.environ.setdefault("CARGO_BUILD_JOBS", "2")
    epoch = source_epoch()
    os.environ.setdefault("SOURCE_DATE_EPOCH", str(epoch))
    own, packages = metadata()
    version = own["version"]
    if not re.fullmatch(r"[0-9A-Za-z.+-]+", version):
        raise RuntimeError("Unsafe package version")
    name = f"Authenticator-{version}-windows-x86_64"
    dist = ROOT / "dist"
    dist.mkdir(exist_ok=True)
    if not dist.resolve().is_relative_to(ROOT.resolve()) or dist.is_symlink() or (hasattr(dist,"is_junction") and dist.is_junction()):
        raise RuntimeError("Distribution directory must be a real workspace directory")
    archive = (dist / f"{name}.zip").resolve()
    if archive.parent != dist.resolve():
        raise RuntimeError("Archive escaped dist")
    digest_file = archive.with_suffix(".zip.sha256")
    if archive.exists():
        if not replace:
            raise RuntimeError("Existing package preserved; pass --replace to replace a verified prior build")
        if not digest_file.is_file() or digest_file.read_text().split()[0] != sha(archive):
            raise RuntimeError("Existing package differs from its checksum; refusing replacement")
        validate(archive)
        with zipfile.ZipFile(archive) as old:
            previous = json.loads(old.read(f"{name}/build-info.json"))
            if previous.get("application") != APP or previous.get("version") != version or previous.get("target") != TARGET:
                raise RuntimeError("Existing package is not an owned matching build")
    if not skip_build:
        command("cargo", "build", "--locked", "--release", "--target", TARGET, "--bin", APP)
    binary = ROOT / "target" / TARGET / "release/Authenticator.exe"
    info = pe_info(binary)
    if any(re.search(r"^(vcruntime|msvcp|concrt).*\.dll$", name, re.I) for name in info["dll_imports"]):
        raise RuntimeError("Static-CRT package unexpectedly imports a VC redistributable DLL")
    stage = scratch / name
    stage.mkdir()
    shutil.copyfile(binary, stage / "Authenticator.exe")
    # Explicit allowlist: no user-data, tests, example binaries or QA captures.
    files = {"README.md":"docs/package-readme.md", "User Guide.md":"docs/user-guide.md", "Security Limits.md":"docs/security-limits.md", "LICENSE-MIT":"LICENSE-MIT", "LICENSE-APACHE":"LICENSE-APACHE"}
    for dest, source in files.items():
        shutil.copyfile(ROOT / source, stage / dest)
    inventory = notices(packages, stage / "licenses")
    write_json(stage / "THIRD-PARTY-NOTICES.json", inventory)
    (stage / "NOTICE.txt").write_text("Authenticator's own source is licensed under MIT OR Apache-2.0.\nOriginal icon: Authenticator contributors.\nBundled dependency, Rust runtime and font declarations/notice texts are indexed\nin THIRD-PARTY-NOTICES.json and licenses/. Both alternatives are retained where supplied;\nAND obligations and font licenses are not collapsed into an MIT-only claim.\nBuild-tool licenses are included conservatively, even when not runtime components.\nMicrosoft Windows system libraries are platform components; no system DLL is shipped.\n",encoding="utf-8")
    record = {"application":APP,"version":version,"target":TARGET,"source_commit":command("git","rev-parse","HEAD"),"source_dirty":bool(command("git","status","--porcelain")),"cargo_lock_sha256":sha(ROOT/"Cargo.lock"),"rustc":command("rustc","--version","--verbose"),"cargo":command("cargo","--version"),"python":sys.version.split()[0],"zlib":zlib.ZLIB_RUNTIME_VERSION,"rustflags":RELEASE_RUSTFLAGS,"source_date_epoch":epoch,"source_timestamp_utc":datetime.fromtimestamp(epoch,timezone.utc).isoformat(),"unsigned":True,"pe":info,"files":[]}
    for path in sorted(stage.rglob("*")):
        if path.is_file():
            record["files"].append({"path":path.relative_to(stage).as_posix(),"sha256":sha(path),"bytes":path.stat().st_size})
    write_json(stage / "build-info.json", record)
    temporary = dist / f".{name}-{os.urandom(8).hex()}.zip.tmp"
    write_archive(stage, name, temporary, epoch)
    validate(temporary)
    os.replace(temporary, archive)
    digest_file.write_text(f"{sha(archive)}  {archive.name}\n",encoding="ascii")
    print(json.dumps({"artifact":str(archive),"sha256":sha(archive),"bytes":archive.stat().st_size,"files":len(record["files"])+1,"dependencies_with_notices":len(inventory),"source_dirty":record["source_dirty"]}))
    return archive

def validate(path: Path) -> None:
    with zipfile.ZipFile(path) as archive:
        entries = archive.infolist()
        if not 1 <= len(entries) <= 5000 or sum(e.file_size for e in entries) > 256*1024*1024 or any(e.file_size > 128*1024*1024 for e in entries):
            raise RuntimeError("Package exceeds validation bounds")
        # zipfile truncates filename at NUL while retaining orig_filename. Inspect
        # the original before any reads, CRC checks or manifest comparisons.
        if any(e.orig_filename != e.filename or any(ord(c) < 32 for c in e.orig_filename) for e in entries):
            raise RuntimeError("Unsafe raw ZIP path")
        names = archive.namelist()
        if len(names) != len(set(n.casefold() for n in names)) or not names:
            raise RuntimeError("Invalid ZIP entries")
        root = names[0].split("/")[0]
        allowed = {"Authenticator.exe", "README.md", "User Guide.md", "Security Limits.md", "LICENSE-MIT", "LICENSE-APACHE", "NOTICE.txt", "THIRD-PARTY-NOTICES.json", "build-info.json"}
        for name in names:
            relative = Path(name)
            if relative.is_absolute() or ".." in relative.parts or ":" in name or "\\" in name or not name.startswith(root+"/"):
                raise RuntimeError("Unsafe ZIP path")
            for part in name.split("/"):
                if not part or part in (".","..") or part.endswith(("."," ")) or any(ord(c)<32 or c in '<>"|?*' for c in part) or re.fullmatch(r"(?i)(CON|PRN|AUX|NUL|CONIN\$|CONOUT\$|COM[1-9¹²³]|LPT[1-9¹²³])",part.partition(".")[0].rstrip(" ")):
                    raise RuntimeError("Unsafe Windows ZIP path")
            if name.lower().endswith((".vault",".authpbk","vault.bin",".pdb",".log")) or "/qa/" in name.lower() or "/examples/" in name.lower():
                raise RuntimeError("Unexpected development/user-data file")
            member = name[len(root)+1:]
            if member not in allowed and not member.startswith("licenses/"):
                raise RuntimeError("File is outside package allowlist")
            if name.endswith("/") or (archive.getinfo(name).external_attr >> 16) & 0o170000 == 0o120000:
                raise RuntimeError("Directories/links are not package payloads")
        folded = {name.casefold() for name in names}
        for name in folded:
            parts = name.split("/")
            if any("/".join(parts[:end]) in folded for end in range(1, len(parts))):
                raise RuntimeError("ZIP file/ancestor collision")
        if archive.testzip() is not None:
            raise RuntimeError("ZIP CRC failed")
        if not {root+"/"+member for member in allowed}.issubset(names):
            raise RuntimeError("Required package payload missing")
        record = json.loads(archive.read(root+"/build-info.json"))
        if not re.fullmatch(r"[0-9A-Za-z.+-]+", record["version"]) or root != f"Authenticator-{record['version']}-windows-x86_64":
            raise RuntimeError("Package version/root mismatch")
        expected = {root+"/build-info.json"}
        for item in record["files"]:
            name = root+"/"+item["path"]
            data = archive.read(name)
            if hashlib.sha256(data).hexdigest() != item["sha256"] or len(data) != item["bytes"]:
                raise RuntimeError("Manifest checksum mismatch")
            expected.add(name)
        if set(names) != expected or len(record["files"])+1 != len(expected) or record["application"] != APP or record["target"] != TARGET:
            raise RuntimeError("Package manifest does not match archive")

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--replace", action="store_true")
    parser.add_argument("--skip-build", action="store_true", help="Use an already-built release binary (local checks only)")
    parser.add_argument("--validate", type=Path)
    options = parser.parse_args()
    if options.validate:
        validate(options.validate)
        print("Package archive paths, CRCs and manifest checksums passed")
    else:
        build(options.replace, options.skip_build)
