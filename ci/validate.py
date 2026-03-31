#!/usr/bin/env python3

import json
import os
import sys

SKILLS_DIR = "agent-skills"
MAX_FILE_BYTES = 1024 * 1024
VALID_CATEGORIES = {"project", "platform", "utility"}


def _parse_yaml(text):
    result = {}
    lines = text.split("\n")
    i = 0
    while i < len(lines):
        line = lines[i]
        if not line.strip() or line.lstrip().startswith("#"):
            i += 1
            continue
        indent = len(line) - len(line.lstrip())
        if indent == 0 and ":" in line:
            key, _, rest = line.partition(":")
            key = key.strip()
            rest = rest.strip()
            if rest in (">", "|"):
                parts = []
                i += 1
                while i < len(lines) and (not lines[i] or lines[i][:1] in (" ", "\t")):
                    parts.append(lines[i].strip())
                    i += 1
                result[key] = " ".join(p for p in parts if p)
                continue
            elif rest == "":
                children = {}
                i += 1
                while i < len(lines) and lines[i] and lines[i][:1] in (" ", "\t"):
                    child = lines[i].strip()
                    if ":" in child:
                        ck, _, cv = child.partition(":")
                        children[ck.strip()] = cv.strip()
                    i += 1
                result[key] = children
                continue
            else:
                result[key] = rest
        i += 1
    return result


def parse_frontmatter(path):
    with open(path, encoding="utf-8") as f:
        text = f.read()
    if not text.startswith("---"):
        return None, "missing opening ---"
    close = text.find("\n---", 3)
    if close == -1:
        return None, "frontmatter not closed"
    return _parse_yaml(text[4:close]), None


def validate_marketplace(skills_dir):
    errors = []
    manifest_path = os.path.join(skills_dir, ".claude-plugin", "marketplace.json")

    if not os.path.isfile(manifest_path):
        return set(), [f".claude-plugin/marketplace.json: file not found"]

    try:
        with open(manifest_path, encoding="utf-8") as f:
            manifest = json.load(f)
    except json.JSONDecodeError as e:
        return set(), [f".claude-plugin/marketplace.json: invalid JSON — {e}"]

    if not isinstance(manifest.get("name"), str) or not manifest["name"]:
        errors.append(".claude-plugin/marketplace.json: missing required field 'name'")

    owner = manifest.get("owner")
    if not isinstance(owner, dict) or not isinstance(owner.get("name"), str) or not owner["name"]:
        errors.append(".claude-plugin/marketplace.json: missing required field 'owner.name'")

    plugins = manifest.get("plugins")
    if not isinstance(plugins, list):
        errors.append(".claude-plugin/marketplace.json: 'plugins' must be a list")
        return set(), errors

    manifest_names = set()
    for i, entry in enumerate(plugins):
        if not isinstance(entry, dict):
            errors.append(f".claude-plugin/marketplace.json: plugins[{i}] must be an object")
            continue
        name = entry.get("name")
        source = entry.get("source")
        if not isinstance(name, str) or not name:
            errors.append(f".claude-plugin/marketplace.json: plugins[{i}] missing 'name'")
        else:
            manifest_names.add(name)
        if not isinstance(source, str) or not source.startswith("./"):
            errors.append(
                f".claude-plugin/marketplace.json: plugins[{i}] 'source' must be a string starting with './' "
            )

    return manifest_names, errors


def validate_plugin_manifest(plugin_name, plugin_path):
    errors = []
    manifest_path = os.path.join(plugin_path, ".claude-plugin", "plugin.json")

    if not os.path.isfile(manifest_path):
        return [f"{plugin_name}/.claude-plugin/plugin.json: file not found"]

    try:
        with open(manifest_path, encoding="utf-8") as f:
            manifest = json.load(f)
    except json.JSONDecodeError as e:
        return [f"{plugin_name}/.claude-plugin/plugin.json: invalid JSON — {e}"]

    name = manifest.get("name")
    if not isinstance(name, str) or not name:
        errors.append(f"{plugin_name}/plugin.json: missing required field 'name'")
    elif name != plugin_name:
        errors.append(
            f"{plugin_name}/plugin.json: 'name' is '{name}' but must match directory name '{plugin_name}'"
        )

    if not isinstance(manifest.get("description"), str) or not manifest["description"]:
        errors.append(f"{plugin_name}/plugin.json: missing required field 'description'")

    if not isinstance(manifest.get("version"), str) or not manifest["version"]:
        errors.append(f"{plugin_name}/plugin.json: missing required field 'version'")

    category = manifest.get("category")
    if not category:
        errors.append(f"{plugin_name}/plugin.json: missing required field 'category'")
    elif category not in VALID_CATEGORIES:
        errors.append(
            f"{plugin_name}/plugin.json: 'category' is '{category}' but must be one of: "
            + ", ".join(sorted(VALID_CATEGORIES))
        )

    keywords = manifest.get("keywords")
    if keywords is not None:
        if not isinstance(keywords, list) or not all(isinstance(k, str) for k in keywords):
            errors.append(f"{plugin_name}/plugin.json: 'keywords' must be a list of strings")

    scope = manifest.get("scope")
    if scope is not None:
        if not isinstance(scope, list) or not all(isinstance(s, str) for s in scope):
            errors.append(f"{plugin_name}/plugin.json: 'scope' must be a list of strings")

    default = manifest.get("default")
    if default is not None and not isinstance(default, bool):
        errors.append(f"{plugin_name}/plugin.json: 'default' must be a boolean")

    return errors


def validate_skill(plugin_name, skill_path):
    errors = []
    skill_md = os.path.join(skill_path, "SKILL.md")

    if not os.path.isfile(skill_md):
        return [f"{plugin_name}: missing skills/{plugin_name}/SKILL.md"]

    for dirpath, _, filenames in os.walk(skill_path):
        for fname in filenames:
            fpath = os.path.join(dirpath, fname)
            size = os.path.getsize(fpath)
            if size > MAX_FILE_BYTES:
                rel = os.path.relpath(fpath, os.path.join(SKILLS_DIR, "plugins"))
                errors.append(f"{rel}: {size} bytes exceeds 1 MB limit")

    fm, err = parse_frontmatter(skill_md)
    if err:
        return errors + [f"{plugin_name}/SKILL.md: {err}"]

    if not fm.get("description"):
        errors.append(f"{plugin_name}/SKILL.md: missing required field 'description'")

    return errors


def main():
    if not os.path.isdir(SKILLS_DIR):
        print(f"ERROR: '{SKILLS_DIR}' directory not found", file=sys.stderr)
        sys.exit(1)

    all_errors = []

    manifest_names, marketplace_errors = validate_marketplace(SKILLS_DIR)
    all_errors.extend(marketplace_errors)

    plugins_dir = os.path.join(SKILLS_DIR, "plugins")
    if not os.path.isdir(plugins_dir):
        all_errors.append(f"'{plugins_dir}' directory not found")
        for err in all_errors:
            print(f"ERROR: {err}", file=sys.stderr)
        sys.exit(1)

    plugin_names = sorted(
        entry for entry in os.listdir(plugins_dir)
        if os.path.isdir(os.path.join(plugins_dir, entry))
    )

    for name in plugin_names:
        if name not in manifest_names:
            print(f"WARNING: plugin '{name}' exists on disk but is not listed in marketplace.json", file=sys.stderr)

    for name in manifest_names:
        if name not in plugin_names:
            all_errors.append(f"marketplace.json lists plugin '{name}' but plugins/{name}/ does not exist")

    for name in plugin_names:
        plugin_path = os.path.join(plugins_dir, name)
        all_errors.extend(validate_plugin_manifest(name, plugin_path))
        skill_path = os.path.join(plugin_path, "skills", name)
        all_errors.extend(validate_skill(name, skill_path))

    for err in all_errors:
        print(f"ERROR: {err}", file=sys.stderr)

    if all_errors:
        sys.exit(1)


if __name__ == "__main__":
    main()
