#!/usr/bin/env python3
"""Admit only recursive public gitlinks and byte-exact committed source overlays."""
import os
from pathlib import Path
import stat
import subprocess
import sys
import tempfile

OVERLAYS = {
    "vendor/tinycomputer": "tinycomputer-local.patch",
    "vendor/tinycomputer/vendor/tinyinference": "tinyinference-decisions-local.patch",
    "vendor/pioneer-tinyjevclient": "tinyjevclient-local.patch",
}
ROOT = Path(sys.argv[1]).resolve()
MODE = sys.argv[2]
if MODE not in {"before", "after"}:
    raise SystemExit("Provenance mode must be before or after")


def git(repo, *args, env=None):
    clean_env = os.environ.copy()
    for key in ("GIT_DIR", "GIT_WORK_TREE", "GIT_COMMON_DIR", "GIT_INDEX_FILE"):
        clean_env.pop(key, None)
    if env:
        clean_env.update(env)
    return subprocess.run(["git", "-C", str(repo), *args], env=clean_env,
                          check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE).stdout


def tree(repo, ref):
    entries = {}
    for record in git(repo, "ls-tree", "-r", "-z", "--full-tree", ref).split(b"\0"):
        if record:
            metadata, name = record.split(b"\t", 1)
            mode, kind, oid = metadata.split()
            entries[os.fsdecode(name)] = (mode, kind, oid)
    return entries


def matches_bytes(repo, entries):
    # Compare raw files with Git blobs; clean filters/textconv cannot hide edits.
    attributes = entries.get(".gitattributes")
    public_crlf = bool(attributes and attributes[1] == b"blob" and
                       b"*.ps1 text eol=crlf" in git(repo, "cat-file", "blob", attributes[2].decode()).splitlines())
    process = subprocess.Popen(["git", "-C", str(repo), "cat-file", "--batch"],
                               stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                               stderr=subprocess.DEVNULL)
    try:
        for name, (mode, kind, oid) in entries.items():
            if kind != b"blob":
                continue
            path = repo / name
            try:
                info = path.lstat()
                if mode == b"120000":
                    if not stat.S_ISLNK(info.st_mode):
                        return False
                    value = os.fsencode(os.readlink(path))
                else:
                    if not stat.S_ISREG(info.st_mode) or bool(info.st_mode & 0o111) != (mode == b"100755"):
                        return False
                    value = path.read_bytes()
            except OSError:
                return False
            process.stdin.write(oid + b"\n")
            process.stdin.flush()
            header = process.stdout.readline().split()
            if len(header) != 3 or header[1] != b"blob":
                return False
            size = int(header[2])
            content = process.stdout.read(size)
            if process.stdout.read(1) != b"\n":
                return False
            if value != content:
                # The committed fork attributes explicitly request CRLF only
                # for PowerShell. Admit that exact checkout transformation,
                # never local attributes, clean filters or arbitrary changes.
                if not (public_crlf and name.endswith(".ps1") and b"\r" not in content and
                        value == content.replace(b"\n", b"\r\n")):
                    return False
        return True
    finally:
        process.stdin.close()
        process.stdout.close()
        process.wait()


def check_untracked(repo):
    ordinary = git(repo, "ls-files", "--others", "--exclude-standard", "-z")
    if ordinary:
        raise ValueError("Untracked source is not eligible for a public package")
    ignored = git(repo, "ls-files", "--others", "--ignored", "--exclude-standard", "-z")
    for raw in ignored.split(b"\0"):
        if not raw:
            continue
        path = Path(os.fsdecode(raw))
        if "target" in path.parts or "node_modules" in path.parts:
            continue
        if repo == ROOT and (path.as_posix().startswith("pioneer/artifacts/") or
                             path.as_posix().startswith("pioneer/local-artifacts/")):
            continue
        if (path.parts[0] in {"src", "crates", "include", "scripts", "pioneer", ".cargo"} or
                path.suffix in {".rs", ".c", ".h", ".cc", ".cpp", ".proto"} or
                path.name in {"Cargo.toml", "Cargo.lock", "build.rs"}):
            raise ValueError("Ignored compiler source/config is not eligible for a public package")


def check_repo(repo, relative, expected):
    if repo.is_symlink() or not (repo / ".git").exists():
        raise ValueError("Recursive dependency must be an initialized repository")
    if Path(os.fsdecode(git(repo, "rev-parse", "--show-toplevel")).strip()).resolve() != repo.resolve():
        raise ValueError("Recursive dependency resolves to another repository")
    head = git(repo, "rev-parse", "HEAD").strip()
    if expected is not None and head != expected:
        raise ValueError("Recursive dependency revision differs from its public gitlink")
    base_tree = git(repo, "rev-parse", "HEAD^{tree}").strip()
    base = tree(repo, base_tree.decode())
    allowed = base_tree
    if relative in OVERLAYS:
        with tempfile.TemporaryDirectory(prefix="pioneer-public-index-") as directory:
            env = {"GIT_INDEX_FILE": str(Path(directory) / "index")}
            git(repo, "read-tree", head.decode(), env=env)
            git(repo, "apply", "--cached", str(ROOT / "pioneer" / OVERLAYS[relative]), env=env)
            allowed = git(repo, "write-tree", env=env).strip()
    index = git(repo, "write-tree").strip()
    if index not in {base_tree, allowed}:
        raise ValueError("Tracked staged source differs from its public tree/overlay")
    check_untracked(repo)
    patched = tree(repo, allowed.decode())
    valid = matches_bytes(repo, patched)
    if not valid and MODE == "before":
        valid = matches_bytes(repo, base)
    if not valid:
        raise ValueError("Tracked source differs from its exact public tree/overlay")
    for name, (mode, kind, oid) in base.items():
        if mode == b"160000":
            child = repo / name
            child_relative = (Path(relative) / name).as_posix() if relative else name
            check_repo(child, child_relative, oid)


try:
    check_repo(ROOT, "", None)
except (ValueError, subprocess.CalledProcessError) as error:
    message = str(error) if isinstance(error, ValueError) else "Public source/overlay verification failed"
    raise SystemExit(message)
print("Recursive public source and approved overlays verified")
