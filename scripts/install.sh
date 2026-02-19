#!/usr/bin/env bash
set -euo pipefail

# スキルをターゲットプロジェクトにインストールするスクリプト
# Agent Skills標準に準拠: .claude/skills/ にコピー

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SKILLS_DIR="$REPO_ROOT/skills"
PROFILES_DIR="$REPO_ROOT/profiles"

# デフォルト値
TARGET=""
PROFILE=""
SKILL=""
SKILLS_CSV=""
ACTION="install"
INSTALL_DIR=".claude/skills"

usage() {
  cat <<EOF
Usage: $0 [OPTIONS]

Options:
  --profile <name>       プロファイル指定でインストール
  --skill <name>         単一スキルをインストール
  --skills "a,b,c"       複数スキルをカンマ区切りでインストール
  --target <path>        インストール先プロジェクトのパス (必須)
  --dir <path>           スキル配置先 (デフォルト: .claude/skills)
  --list                 インストール済みスキルを一覧表示
  --available            利用可能なスキル/プロファイルを一覧表示
  --uninstall <name>     スキルをアンインストール
  --dry-run              実際にはコピーせずプレビューのみ
  -h, --help             ヘルプを表示

Examples:
  $0 --profile web-frontend --target ~/my-project
  $0 --skill code-review --target ~/my-project
  $0 --skills "code-review,test-gen" --target ~/my-project
  $0 --list --target ~/my-project
EOF
  exit 0
}

DRY_RUN=false

while [ $# -gt 0 ]; do
  case "$1" in
    --profile)    PROFILE="$2";     shift 2 ;;
    --skill)      SKILL="$2";       shift 2 ;;
    --skills)     SKILLS_CSV="$2";  shift 2 ;;
    --target)     TARGET="$2";      shift 2 ;;
    --dir)        INSTALL_DIR="$2"; shift 2 ;;
    --list)       ACTION="list";    shift ;;
    --available)  ACTION="available"; shift ;;
    --uninstall)  ACTION="uninstall"; SKILL="$2"; shift 2 ;;
    --dry-run)    DRY_RUN=true;     shift ;;
    -h|--help)    usage ;;
    *)            echo "Unknown option: $1"; usage ;;
  esac
done

# --- available: レジストリの内容表示 ---
if [ "$ACTION" = "available" ]; then
  echo "📦 Available Skills:"
  echo ""
  for skill_dir in "$SKILLS_DIR"/*/; do
    [ -d "$skill_dir" ] || continue
    name=$(basename "$skill_dir")
    desc=""
    if [ -f "$skill_dir/SKILL.md" ]; then
      # フロントマターから description を抽出
      desc=$(sed -n '/^---$/,/^---$/{ /^description:/{ s/^description: *//; s/^> *//; p; } }' "$skill_dir/SKILL.md" | head -1)
    fi
    printf "  %-24s %s\n" "$name" "$desc"
  done

  echo ""
  echo "📋 Available Profiles:"
  echo ""
  for profile_file in "$PROFILES_DIR"/*.json; do
    [ -f "$profile_file" ] || continue
    name=$(basename "$profile_file" .json)
    desc=$(python3 -c "import json; print(json.load(open('$profile_file')).get('description',''))" 2>/dev/null || echo "")
    printf "  %-24s %s\n" "$name" "$desc"
  done
  exit 0
fi

# --- 以降は target 必須 ---
if [ -z "$TARGET" ]; then
  echo "Error: --target is required"
  echo "Run with --help for usage"
  exit 1
fi

TARGET_SKILLS="$TARGET/$INSTALL_DIR"

# --- list: インストール済み一覧 ---
if [ "$ACTION" = "list" ]; then
  if [ ! -d "$TARGET_SKILLS" ]; then
    echo "No skills installed at $TARGET_SKILLS"
    exit 0
  fi
  echo "📦 Installed skills in $TARGET_SKILLS:"
  echo ""
  for skill_dir in "$TARGET_SKILLS"/*/; do
    [ -d "$skill_dir" ] || continue
    echo "  $(basename "$skill_dir")"
  done
  exit 0
fi

# --- uninstall ---
if [ "$ACTION" = "uninstall" ]; then
  if [ -z "$SKILL" ]; then
    echo "Error: specify skill name to uninstall"
    exit 1
  fi
  target_path="$TARGET_SKILLS/$SKILL"
  if [ ! -d "$target_path" ]; then
    echo "Skill '$SKILL' is not installed at $TARGET_SKILLS"
    exit 1
  fi
  rm -rf "$target_path"
  echo "🗑️  Uninstalled: $SKILL"
  exit 0
fi

# --- install: スキル一覧を解決 ---
INSTALL_LIST=()

if [ -n "$PROFILE" ]; then
  profile_file="$PROFILES_DIR/$PROFILE.json"
  if [ ! -f "$profile_file" ]; then
    echo "Error: Profile '$PROFILE' not found at $profile_file"
    exit 1
  fi
  # JSON からスキル一覧を取得
  mapfile -t profile_skills < <(python3 -c "
import json
data = json.load(open('$profile_file'))
for s in data.get('skills', []):
    print(s)
")
  INSTALL_LIST+=("${profile_skills[@]}")
fi

if [ -n "$SKILL" ] && [ "$ACTION" = "install" ]; then
  INSTALL_LIST+=("$SKILL")
fi

if [ -n "$SKILLS_CSV" ]; then
  IFS=',' read -ra csv_skills <<< "$SKILLS_CSV"
  INSTALL_LIST+=("${csv_skills[@]}")
fi

if [ ${#INSTALL_LIST[@]} -eq 0 ]; then
  echo "Error: No skills specified. Use --profile, --skill, or --skills"
  exit 1
fi

# 重複除去
mapfile -t INSTALL_LIST < <(printf '%s\n' "${INSTALL_LIST[@]}" | sort -u)

# --- 実行 ---
echo "🔧 Installing ${#INSTALL_LIST[@]} skill(s) to $TARGET_SKILLS"
echo ""

for skill_name in "${INSTALL_LIST[@]}"; do
  src="$SKILLS_DIR/$skill_name"
  dst="$TARGET_SKILLS/$skill_name"

  if [ ! -d "$src" ]; then
    echo "  ⚠️  Skip: '$skill_name' not found in registry"
    continue
  fi

  if [ "$DRY_RUN" = true ]; then
    echo "  [dry-run] Would install: $skill_name"
    continue
  fi

  mkdir -p "$dst"
  # コピー（.gitkeep は除外）
  cp -a "$src/." "$dst/"
  find "$dst" -name '.gitkeep' -delete 2>/dev/null || true
  echo "  ✅ $skill_name"
done

echo ""
if [ "$DRY_RUN" = true ]; then
  echo "Dry run complete. No files were copied."
else
  echo "Done! Skills installed to $TARGET_SKILLS"
fi
