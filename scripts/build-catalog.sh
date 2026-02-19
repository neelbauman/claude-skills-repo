#!/usr/bin/env bash
set -euo pipefail

# skills/ 配下の全スキルからメタデータを抽出し catalog.json を生成する

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SKILLS_DIR="$REPO_ROOT/skills"
CATALOG_FILE="$REPO_ROOT/catalog.json"

python3 - "$SKILLS_DIR" "$CATALOG_FILE" <<'PYTHON_SCRIPT'
import sys, os, json, re, glob
from pathlib import Path
from datetime import datetime, timezone

skills_dir = Path(sys.argv[1])
catalog_file = Path(sys.argv[2])

def parse_frontmatter(skill_md_path):
    """SKILL.md の YAML フロントマターを簡易パースする"""
    text = skill_md_path.read_text(encoding="utf-8")
    match = re.match(r'^---\s*\n(.*?)\n---', text, re.DOTALL)
    if not match:
        return {}
    fm = {}
    current_key = None
    for line in match.group(1).splitlines():
        # multiline value continuation (e.g. description: >)
        if current_key and line.startswith("  "):
            fm[current_key] = (fm.get(current_key, "") + " " + line.strip()).strip()
            continue
        kv = re.match(r'^(\w[\w-]*):\s*(.*)', line)
        if kv:
            key, val = kv.group(1), kv.group(2).strip()
            if val == ">" or val == "|":
                current_key = key
                fm[key] = ""
            else:
                current_key = key
                fm[key] = val
        else:
            current_key = None
    return fm

def get_skill_files(skill_dir):
    """スキルに含まれるファイル一覧を返す"""
    files = []
    for f in sorted(skill_dir.rglob("*")):
        if f.is_file() and f.name != ".gitkeep":
            files.append(str(f.relative_to(skill_dir)))
    return files

catalog = {
    "version": "1.0.0",
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "skills": []
}

for skill_dir in sorted(skills_dir.iterdir()):
    if not skill_dir.is_dir():
        continue
    skill_md = skill_dir / "SKILL.md"
    if not skill_md.exists():
        print(f"  ⚠️  Skip: {skill_dir.name}/ (no SKILL.md)", file=sys.stderr)
        continue

    fm = parse_frontmatter(skill_md)
    name = fm.get("name", skill_dir.name)
    description = fm.get("description", "")

    entry = {
        "name": name,
        "dir": skill_dir.name,
        "description": description,
        "files": get_skill_files(skill_dir),
    }

    # optional fields
    if "tags" in fm:
        entry["tags"] = [t.strip() for t in fm["tags"].split(",")]
    if "dependencies" in fm:
        entry["dependencies"] = fm["dependencies"]

    catalog["skills"].append(entry)

catalog_file.write_text(json.dumps(catalog, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
print(f"✅ Generated {catalog_file} with {len(catalog['skills'])} skill(s)")
PYTHON_SCRIPT
