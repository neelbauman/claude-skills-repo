#!/usr/bin/env bash
set -euo pipefail

# 新規スキルの雛形を生成するスクリプト
# Usage: ./scripts/new-skill.sh <skill-name> [--description "説明文"]

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TEMPLATE="$REPO_ROOT/templates/SKILL.md.template"

if [ $# -lt 1 ]; then
  echo "Usage: $0 <skill-name> [--description \"説明文\"]"
  echo ""
  echo "Example:"
  echo "  $0 code-review --description \"コードレビューを体系的に実施する\""
  exit 1
fi

SKILL_NAME="$1"
DESCRIPTION="TODO: このスキルの説明を書く。Claudeがいつこのスキルを使うべきか判断できるように具体的に。"

shift
while [ $# -gt 0 ]; do
  case "$1" in
    --description)
      DESCRIPTION="$2"
      shift 2
      ;;
    *)
      echo "Unknown option: $1"
      exit 1
      ;;
  esac
done

SKILL_DIR="$REPO_ROOT/skills/$SKILL_NAME"

if [ -d "$SKILL_DIR" ]; then
  echo "Error: skills/$SKILL_NAME already exists"
  exit 1
fi

# ディレクトリ構成を作成
mkdir -p "$SKILL_DIR"/{scripts,references,assets}

# テンプレートからSKILL.mdを生成
sed \
  -e "s|{{SKILL_NAME}}|$SKILL_NAME|g" \
  -e "s|{{DESCRIPTION}}|$DESCRIPTION|g" \
  "$TEMPLATE" > "$SKILL_DIR/SKILL.md"

# 空の .gitkeep を配置（空ディレクトリ保持用）
touch "$SKILL_DIR/scripts/.gitkeep"
touch "$SKILL_DIR/references/.gitkeep"
touch "$SKILL_DIR/assets/.gitkeep"

echo "✅ Created skills/$SKILL_NAME/"
echo ""
echo "  skills/$SKILL_NAME/"
echo "  ├── SKILL.md          ← ここを編集"
echo "  ├── scripts/"
echo "  ├── references/"
echo "  └── assets/"
echo ""
echo "Next steps:"
echo "  1. skills/$SKILL_NAME/SKILL.md を編集"
echo "  2. ./scripts/build-catalog.sh でカタログ更新"
