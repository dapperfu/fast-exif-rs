#!/usr/bin/env python3

"""Git-based versioning for Python packages.

This module provides a way to automatically determine version numbers
based on git tags and commits.
"""

import os
import re
import subprocess
import sys
from typing import Dict, Optional, Tuple


def get_keywords() -> Dict[str, str]:
    """Get version keywords from git."""
    # These will be replaced by git during git-archive
    git_refnames = "$Format:%d$"
    git_full = "$Format:%H$"
    git_date = "$Format:%ci$"
    
    keywords = {
        "refnames": git_refnames,
        "full": git_full,
        "date": git_date,
    }
    return keywords


def run_command(commands: list, cwd: Optional[str] = None) -> Tuple[int, str, str]:
    """Run a command and return (returncode, stdout, stderr)."""
    assert isinstance(commands, list)
    p = subprocess.Popen(
        commands,
        cwd=cwd,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    stdout, stderr = p.communicate()
    return p.returncode, stdout.decode("utf-8"), stderr.decode("utf-8")


def versions_from_parentdir(parentdir_prefix: str, root: str, verbose: bool) -> Dict[str, str]:
    """Try to determine the version from the parent directory name."""
    if not parentdir_prefix:
        return {}
    
    dirname = os.path.basename(root)
    if not dirname.startswith(parentdir_prefix):
        if verbose:
            print(f"guessing rootdir is '{root}', but '{dirname}' doesn't start with prefix '{parentdir_prefix}'")
        return {}
    
    return {"version": dirname[len(parentdir_prefix):], "full-revisionid": None}


def git_get_keywords(versionfile_abs: str) -> Dict[str, str]:
    """Get version information from git keywords."""
    # the code embedded in _version.py can just fetch the value of these
    # keywords. When used from setup.py, we don't want to import _version.py,
    # so we do it with a regexp instead. This function is not used from
    # _version.py.
    keywords = {}
    try:
        f = open(versionfile_abs, "r")
        for line in f.readlines():
            if line.strip().startswith("git_refnames ="):
                mo = re.search(r'=\s*"(.*)"', line)
                if mo:
                    keywords["refnames"] = mo.group(1)
            if line.strip().startswith("git_full ="):
                mo = re.search(r'=\s*"(.*)"', line)
                if mo:
                    keywords["full"] = mo.group(1)
            if line.strip().startswith("git_date ="):
                mo = re.search(r'=\s*"(.*)"', line)
                if mo:
                    keywords["date"] = mo.group(1)
        f.close()
    except EnvironmentError:
        pass
    return keywords


def git_versions_from_keywords(keywords: Dict[str, str], tag_prefix: str, verbose: bool) -> Dict[str, str]:
    """Get version information from git keywords."""
    if not keywords:
        return {}
    
    refnames = keywords["refnames"].strip()
    if refnames.startswith("$Format"):
        if verbose:
            print("keywords are unexpanded, not using")
        return {}
    
    refs = set([r.strip() for r in refnames.strip("()").split(",")])
    TAG = f"refs/tags/{tag_prefix}"
    
    # starting in git-1.8.3, tags are listed as "refs/tags/1.0" instead of
    # just "1.0". If we see a "refs/tags/" prefix, prefer those.
    TAG = f"refs/tags/{tag_prefix}"
    tags = set([r[len(TAG):] for r in refs if r.startswith(TAG)])
    if not tags:
        # Either we're using git < 1.8.3, or there really are no tags. We use
        # a heuristic: assume all version tags have a digit. The old git %d
        # expansion behaves like git log --decorate=short and strips out the
        # refs/heads/ and refs/tags/ prefixes from the refnames returned.
        # so we will look for "0.1" instead of "refs/tags/0.1".
        TAG = tag_prefix
        tags = set([r[len(TAG):] for r in refs if r.startswith(TAG)])
        # ... and also look for "v0.1" instead of "refs/tags/v0.1"
        TAG = f"refs/tags/{tag_prefix}"
        tags = tags.union(set([r[len(TAG):] for r in refs if r.startswith(TAG)]))
    
    if verbose:
        print(f"discarding '{refs - tags}', no '{TAG}' prefix")
    
    if not tags:
        return {}
    
    # prefer the latest tag
    tags = sorted(tags)
    tag = tags[-1]
    
    # now we have a tag, extract the version
    version = tag
    if verbose:
        print(f"picked {version}")
    
    return {"version": version, "full-revisionid": keywords.get("full")}


def git_versions_from_vcs(tag_prefix: str, root: str, verbose: bool) -> Dict[str, str]:
    """Get version information from git."""
    # this runs 'git' from the root of the source tree. This only gets called
    # if the git-archive 'subst' keywords were *not* expanded, and
    # _version.py hasn't already been rewritten with a short version string,
    # meaning we're inside a checked out source tree.

    if not os.path.exists(os.path.join(root, ".git")):
        if verbose:
            print("no .git in {root}")
        return {}

    GITS = ["git"]
    if sys.platform == "win32":
        GITS = ["git.cmd", "git.exe"]
    
    for git in GITS:
        try:
            out, rc = run_command([git, "describe", "--tags", "--dirty", "--always"], cwd=root)
            if rc != 0:
                continue
            if verbose:
                print(f"git describe output: {out}")
            # strip initial 'v' and trailing '-dirty' if present
            version = out.strip()
            if version.startswith(tag_prefix):
                version = version[len(tag_prefix):]
            if version.endswith("-dirty"):
                version = version[:-6]
            return {"version": version, "full-revisionid": None}
        except EnvironmentError:
            continue
    
    return {}


def get_versions() -> Dict[str, str]:
    """Get version information."""
    # I am in _version.py, which lives at ROOT/VERSIONFILE_SOURCE. If we have
    # __file__, we can work backwards from there to the root. Some
    # py2exe/bbfreeze/non-CPython implementations don't do __file__, in which
    # case we can only use expanded keywords.

    keywords = {"refnames": "", "full": "", "date": ""}
    ver = run_command(["git", "describe", "--tags", "--dirty", "--always"])
    if ver[0] == 0:
        keywords["refnames"] = ver[1].strip()
    ver = run_command(["git", "rev-parse", "HEAD"])
    if ver[0] == 0:
        keywords["full"] = ver[1].strip()
    ver = run_command(["git", "log", "-1", "--format=%ci"])
    if ver[0] == 0:
        keywords["date"] = ver[1].strip()

    refnames = keywords["refnames"].strip()
    if refnames.startswith("$Format"):
        if verbose:
            print("keywords are unexpanded, not using")
        return {"version": "0+unknown", "full-revisionid": keywords["full"]}

    # starting in git-1.8.3, tags are listed as "refs/tags/1.0" instead of
    # just "1.0". If we see a "refs/tags/" prefix, prefer those.
    TAG = "refs/tags/v"
    tags = set([r[len(TAG):] for r in refnames.strip("()").split(",") if r.startswith(TAG)])
    if not tags:
        # Either we're using git < 1.8.3, or there really are no tags. We use
        # a heuristic: assume all version tags have a digit. The old git %d
        # expansion behaves like git log --decorate=short and strips out the
        # refs/heads/ and refs/tags/ prefixes from the refnames returned.
        # so we will look for "0.1" instead of "refs/tags/0.1".
        TAG = "v"
        tags = set([r[len(TAG):] for r in refnames.strip("()").split(",") if r.startswith(TAG)])

    if not tags:
        return {"version": "0+unknown", "full-revisionid": keywords["full"]}

    # prefer the latest tag
    tags = sorted(tags)
    tag = tags[-1]

    # now we have a tag, extract the version
    version = tag

    # look for -dirty suffix
    if refnames.endswith("-dirty"):
        version += "+dirty"

    return {"version": version, "full-revisionid": keywords["full"]}


def get_version() -> str:
    """Get the version string."""
    return get_versions()["version"]


def get_cmdclass() -> Dict[str, type]:
    """Get custom distutils/setuptools command classes."""
    return {}


def get_versionfile_source() -> str:
    """Get the version file source path."""
    return "fast_exif_reader/_version.py"


def write_to_version_file(filename: str, versions: Dict[str, str]) -> None:
    """Write version information to a file."""
    os.makedirs(os.path.dirname(filename), exist_ok=True)
    
    with open(filename, "w") as f:
        f.write("""# This file was generated by versioneer.py
# from git metadata, do not edit!

""" + f'''git_refnames = "{versions.get("refnames", "")}"
git_full = "{versions.get("full", "")}"
git_date = "{versions.get("date", "")}"
version = "{versions.get("version", "0+unknown")}"
full_revisionid = "{versions.get("full-revisionid", "")}"
dirty = {versions.get("dirty", False)}
''')


if __name__ == "__main__":
    print(get_version())
