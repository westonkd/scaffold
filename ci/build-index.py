#!/usr/bin/env python3

import json
import os
import sys
from datetime import datetime, timezone

SKILLS_DIR = "agent-skills"


def iso_utc(ts):
    return datetime.fromtimestamp(ts, tz=timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def main():
    plugins_dir = os.path.join(SKILLS_DIR, "plugins")
    if not os.path.isdir(plugins_dir):
        print(f"ERROR: '{plugins_dir}' directory not found", file=sys.stderr)
        sys.exit(1)

    skills = []
    for plugin_name in sorted(os.listdir(plugins_dir)):
        plugin_path = os.path.join(plugins_dir, plugin_name)
        if not os.path.isdir(plugin_path):
            continue

        manifest_path = os.path.join(plugin_path, ".claude-plugin", "plugin.json")
        skill_md = os.path.join(plugin_path, "skills", plugin_name, "SKILL.md")

        if not os.path.isfile(manifest_path):
            print(f"WARNING: skipping {plugin_name} — missing .claude-plugin/plugin.json", file=sys.stderr)
            continue
        if not os.path.isfile(skill_md):
            print(f"WARNING: skipping {plugin_name} — missing skills/{plugin_name}/SKILL.md", file=sys.stderr)
            continue

        try:
            with open(manifest_path, encoding="utf-8") as f:
                manifest = json.load(f)
        except (json.JSONDecodeError, OSError) as e:
            print(f"WARNING: could not parse plugin.json for {plugin_name}: {e}", file=sys.stderr)
            continue

        skill = {
            "name": manifest.get("name", plugin_name),
            "description": manifest.get("description", ""),
            "type": manifest.get("category", ""),
            "status": manifest.get("status", "active"),
            "tags": manifest.get("keywords", []),
            "scope": manifest.get("scope", []),
            "updated_at": iso_utc(os.path.getmtime(skill_md)),
        }

        if manifest.get("default") is True:
            skill["default"] = True

        skills.append(skill)

    index = {
        "version": 1,
        "updated_at": datetime.now(tz=timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "skills": skills,
    }

    print(json.dumps(index, indent=2))


if __name__ == "__main__":
    main()
