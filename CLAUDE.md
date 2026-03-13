# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Claude Skills Registry — トピック志向で Agent Skills, Agents, Hooks, Profiles を管理する CLI ツールとレジストリ。
`claude-registry` コマンドでスキル等をプロジェクトや `~/.claude` にインストールする。

## Build & Development

```bash
# ビルド（Rust CLI）
cd cli && cargo build --release
# バイナリ: cli/target/release/claude-registry

# テスト
cd cli && cargo test

# 単一テスト
cd cli && cargo test <test_name>

# フォーマット & Lint
cd cli && cargo fmt --check
cd cli && cargo clippy

# カタログ再生成（スキル/エージェント追加・変更後に実行）
claude-registry catalog build
```

## Architecture

### CLI (Rust) — `cli/src/`

コマンドルーティング型アーキテクチャ。clap derive マクロでサブコマンドを定義。

- `main.rs` — エントリポイント、コマンドディスパッチ
- `cli.rs` — clap によるコマンド定義（skill / agent / hook / profile / catalog / complete）
- `registry.rs` — レジストリパス解決（`CLAUDE_REGISTRY_ROOT` → `~/.local/share/claude-registry` → カレントディレクトリ）
- `installer.rs` — ファイルコピーロジック
- `catalog_builder.rs` — SKILL.md / エージェント .md からカタログ JSON を自動生成
- `frontmatter.rs` — Markdown ファイルの YAML フロントマター解析
- `commands/` — 各サブコマンドの実装（`skill.rs`, `agent.rs`, `hook.rs`, `profile.rs`, `catalog.rs`, `complete.rs`）

### Registry Content — `claude/`

- `claude/skills/<name>/SKILL.md` — スキル定義（YAML フロントマター + Markdown 本文）。references/ に詳細を分離
- `claude/agents/<name>.md` — エージェント定義（フロントマター: name, description, tools, model）
- `claude/hooks/<name>/HOOK.json` — フック定義。install 時に settings.json へ JSON マージ（`_registry_id` で冪等管理）

### Other Key Directories

- `profiles/` — スキル+エージェントのプリセット（JSON）
- `templates/` — `skill new` / `agent new` / `hook new` で使う雛形
- `specification/` — Doorstop + Gherkin による仕様管理
- `completions/` — シェル補完スクリプト

## Conventions

- **コミット規約**: Conventional Commits を使用。scope は `skills`, `agents`, `profiles`, `cli` など
  - 例: `feat(skills): add new-skill-name skill`, `chore(cli): improve error handling`
- **スキル設計**: 1スキル = 1トピック、SKILL.md は 500行以内、超過分は references/ へ分離
- **フロントマター**: SKILL.md の `description` にはトリガー条件を明記する（Claude がいつ使うか判断するため）
- **命令形**: スキル本文は「〜する」形式で記述（「〜してください」ではなく）
