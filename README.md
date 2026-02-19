# Claude Skills Registry

トピック志向で開発・管理する Agent Skills & Agents リポジトリ。
プロジェクトやタスクに応じて、必要なスキルとエージェントを選んでインストールできます。

## コンセプト

```
┌──────────────────────────────────────────────────┐
│  Skills Registry (このリポジトリ)                 │
│                                                   │
│  claude/skills/                                   │
│    ├── code-review/       ← トピック単位で分割    │
│    ├── git-conventional/                          │
│    └── ...                                        │
│                                                   │
│  claude/agents/                                   │
│    ├── repo-researcher.md ← エージェント定義      │
│    └── ...                                        │
│                                                   │
│  profiles/                                        │
│    ├── web-frontend.json  ← 用途別プリセット      │
│    └── ...                                        │
└──────────────┬───────────────────────────────────┘
               │ claude-registry install
               ▼
┌──────────────────────────┐
│  Target Project          │
│  .claude/skills/         │
│    ├── code-review/      │
│    └── ...               │
│  .claude/agents/         │
│    └── repo-researcher.md│
└──────────────────────────┘
```

## 設計原則

- **Single Responsibility**: 1スキル = 1トピック。肥大化させない
- **Composable**: プロファイルで組み合わせ、プロジェクトに合わせてカスタマイズ
- **Progressive Disclosure**: SKILL.md は軽量に、詳細は references/ に分離
- **Portable**: Agent Skills 標準 (agentskills.io) 準拠

## クイックスタート

### ビルド

```bash
cd cli && cargo build --release
# バイナリ: cli/target/release/claude-registry
```

### スキル操作

```bash
# 利用可能なスキル一覧
claude-registry skill available

# スキルをインストール
claude-registry skill install code-review --target /path/to/project

# 複数スキルを一度にインストール
claude-registry skill install code-review git-conventional --target /path/to/project

# インストール済みスキル一覧
claude-registry skill list --target /path/to/project

# スキルをアンインストール
claude-registry skill uninstall code-review --target /path/to/project

# 新しいスキルを作成
claude-registry skill new my-skill --description "説明文"
```

### エージェント操作

```bash
# 利用可能なエージェント一覧
claude-registry agent available

# エージェントをインストール
claude-registry agent install repo-researcher --target /path/to/project

# エージェントをアンインストール
claude-registry agent uninstall repo-researcher --target /path/to/project
```

### プロファイル操作

```bash
# プロファイル一覧
claude-registry profile list

# プロファイルでスキル＋エージェントを一括インストール
claude-registry profile install web-frontend --target /path/to/project
```

### カタログ生成

```bash
claude-registry catalog build
# → skill-catalog.json, agent-catalog.json が生成される
```

## ディレクトリ構成

```
.
├── claude/
│   ├── skills/              # スキル本体（トピックごとに1ディレクトリ）
│   │   └── <skill-name>/
│   │       ├── SKILL.md
│   │       ├── scripts/
│   │       ├── references/
│   │       └── assets/
│   └── agents/              # エージェント定義（1ファイル = 1エージェント）
│       └── <agent-name>.md
│
├── profiles/                # 用途別プリセット
│   └── <profile-name>.json
│
├── cli/                     # Rust CLI ツール
│   ├── Cargo.toml
│   └── src/
│
├── templates/               # 雛形テンプレート
│   ├── SKILL.md.template
│   └── AGENT.md.template
│
├── skill-catalog.json       # スキルカタログ（自動生成）
├── agent-catalog.json       # エージェントカタログ（自動生成）
│
└── README.md
```

## スキルの作り方

```bash
# 雛形を生成
claude-registry skill new my-new-skill --description "説明文"

# claude/skills/my-new-skill/SKILL.md を編集
# テスト → 反復改善

# カタログを更新
claude-registry catalog build
```

詳しくは [CONTRIBUTING.md](./CONTRIBUTING.md) を参照。

## プロファイルの仕組み

プロファイルは「このプロジェクトにはこのスキル群とエージェントが必要」を定義する JSON です。

```json
{
  "name": "web-frontend",
  "description": "Webフロントエンド開発向けスキルセット",
  "skills": ["code-review", "git-conventional"],
  "agents": ["repo-researcher"]
}
```

自分のプロジェクトに合わせたプロファイルを作って、チームで共有できます。

## ライセンス

MIT
