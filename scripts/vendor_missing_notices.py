"""Development-only: pin missing upstream license texts to published crate commits."""
import hashlib
import json
from pathlib import Path
import runpy
import urllib.error
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
package = runpy.run_path(str(ROOT / "scripts/package_windows.py"))
_, dependencies = package["metadata"]()
repositories = {
    "accesskit":"AccessKit/accesskit", "accesskit_consumer":"AccessKit/accesskit",
    "accesskit_windows":"AccessKit/accesskit", "accesskit_winit":"AccessKit/accesskit",
    "ecolor":"emilk/egui", "eframe":"emilk/egui", "egui":"emilk/egui",
    "egui-winit":"emilk/egui", "egui_glow":"emilk/egui", "emath":"emilk/egui",
    "epaint":"emilk/egui", "epaint_default_fonts":"emilk/egui", "gl_generator":"brendanzab/gl-rs",
    "khronos_api":"brendanzab/gl-rs", "profiling":"aclysma/profiling",
}
cache = {}
heads = {}
release_revisions = {}
for dependency in dependencies:
    if dependency["name"] == "authenticator":
        continue
    existing = package["license_files"](dependency)
    needs_font_code_license = dependency["name"] == "epaint_default_fonts" and not (ROOT / "assets/notices" / f"{dependency['name']}-{dependency['version']}" / "provenance.json").is_file()
    provenance_file = ROOT / "assets/notices" / f"{dependency['name']}-{dependency['version']}" / "provenance.json"
    needs_version_provenance = dependency["name"] in repositories and repositories[dependency["name"]] == "emilk/egui" and provenance_file.is_file() and json.loads(provenance_file.read_text(encoding="utf-8"))["current_upstream_fallback"]
    expression, _ = package["packaged_license"](dependency, existing)
    if existing and not needs_font_code_license and not needs_version_provenance and package["license_choice"](expression,existing) is not None:
        continue
    repo = repositories[dependency["name"]]
    source = Path(dependency["manifest_path"]).parent
    commit = json.loads((source / ".cargo_vcs_info.json").read_text())["git"]["sha1"]
    if len(commit) != 40 or any(c not in "0123456789abcdef" for c in commit):
        raise RuntimeError("Invalid upstream revision")
    destination = ROOT / "assets/notices" / f"{dependency['name']}-{dependency['version']}"
    destination.mkdir(parents=True, exist_ok=True)
    records = []
    revision = commit
    fallback = False
    release_tag = None
    for attempt in range(3):
        for name in ("LICENSE", "LICENSE-APACHE", "LICENSE-MIT", "NOTICE"):
            url = f"https://raw.githubusercontent.com/{repo}/{revision}/{name}"
            if url not in cache:
                try:
                    with urllib.request.urlopen(url, timeout=30) as response:
                        cache[url] = response.read(2*1024*1024+1)
                except urllib.error.HTTPError as error:
                    if error.code != 404:
                        raise
                    cache[url] = None
            content = cache[url]
            if content is not None:
                if len(content) > 2*1024*1024:
                    raise RuntimeError("Oversized notice")
                (destination / name).write_bytes(content)
                records.append({"file":name,"url":url,"sha256":hashlib.sha256(content).hexdigest()})
        if records:
            break
        if attempt == 0 and repo == "emilk/egui":
            tag = dependency["version"]
            key = (repo,tag)
            if key not in release_revisions:
                request = urllib.request.Request(f"https://api.github.com/repos/{repo}/git/ref/tags/{tag}",headers={"User-Agent":"Authenticator-notice-provenance"})
                try:
                    ref = json.load(urllib.request.urlopen(request,timeout=30))["object"]
                    if ref["type"] == "tag":
                        request = urllib.request.Request(ref["url"],headers={"User-Agent":"Authenticator-notice-provenance"})
                        ref = json.load(urllib.request.urlopen(request,timeout=30))["object"]
                    if ref["type"] != "commit" or len(ref["sha"]) != 40 or any(c not in "0123456789abcdef" for c in ref["sha"]):
                        raise RuntimeError("Release tag did not resolve to a commit")
                    release_revisions[key] = ref["sha"]
                except urllib.error.HTTPError as error:
                    if error.code != 404:
                        raise
                    release_revisions[key] = None
            if release_revisions[key]:
                revision = release_revisions[key]
                release_tag = tag
                continue
        if repo not in heads:
            request = urllib.request.Request(f"https://api.github.com/repos/{repo}/commits?per_page=1",headers={"User-Agent":"Authenticator-notice-provenance"})
            heads[repo] = json.load(urllib.request.urlopen(request,timeout=30))[0]["sha"]
        revision = heads[repo]
        release_tag = None
        fallback = True
    if not records:
        raise RuntimeError(f"No upstream license found for {dependency['name']}")
    (destination / "provenance.json").write_text(json.dumps({"crate":dependency["name"],"version":dependency["version"],"published_crate_commit":commit,"license_source_commit":revision,"release_tag":release_tag,"current_upstream_fallback":fallback,"note":"License texts only; source is the published crate's commit, or the recorded release tag when that commit is unavailable, or an explicitly marked current-upstream fallback. This does not establish code-version equality. SPDX declaration is retained from the published crate.","files":records},indent=2)+"\n",encoding="utf-8")
    print(dependency["name"], dependency["version"], len(records))
