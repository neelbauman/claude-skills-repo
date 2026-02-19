# Contributing

## 新しいスキルを追加する

### 1. 雛形を生成

```bash
claude-registry skill new <skill-name> --description "説明文"
```

### 2. SKILL.md を編集

**必須フィールド:**

- `name`: スキル名（小文字、ハイフン区切り、64文字以内）
- `description`: トリガー条件を含む説明（200文字以内推奨）

**オプションフィールド:**

- `tags`: カンマ区切りのタグ（カタログ分類用）

**良い description の条件:**

- Claudeが「いつ使うべきか」判断できる具体的なトリガーワードを含む
- スキルの能力とスコープが明確
- やらないことも暗示できるとベター

❌ 悪い例: `コードを改善するスキル`
✅ 良い例: `コードレビューを体系的に実施する。PRレビュー、コード品質チェック、リファクタリング提案時に使用。`

### 3. 設計原則

- **1スキル = 1トピック**: 「コードレビュー＋テスト生成＋デプロイ」を1つにまとめない
- **500行以内**: SKILL.md が長くなりすぎたら references/ に分割
- **例を含める**: 入力→出力の具体例がClaudeの精度を大きく上げる
- **命令形で書く**: 「〜してください」ではなく「〜する」

### 4. テスト

Claude Code でスキルをインストールして実際に動かす:

```bash
claude-registry skill install my-skill --target ~/test-project
cd ~/test-project
claude  # スキルが認識されるか確認
```

### 5. カタログを更新

```bash
claude-registry catalog build
```

### 6. プロファイルに追加（必要なら）

新しいスキルが既存のプロファイルに合う場合は `profiles/*.json` に追加。

## 新しいエージェントを追加する

### 1. テンプレートを参考に作成

`templates/AGENT.md.template` を参考に `claude/agents/<agent-name>.md` を作成。

### 2. フロントマターの必須フィールド

- `name`: エージェント名
- `description`: エージェントの役割と発動条件
- `tools`: 使用するツール（カンマ区切り）
- `model`: 使用するモデル

### 3. カタログを更新

```bash
claude-registry catalog build
```

## プロファイルを追加する

`profiles/<name>.json` を作成:

```json
{
  "name": "my-profile",
  "description": "このプロファイルの説明",
  "skills": ["skill-a", "skill-b"],
  "agents": ["agent-a"]
}
```

`agents` フィールドはオプショナルです。

## コミット規約

このリポジトリ自体も Conventional Commits を使用:

- `feat(skills): add new-skill-name skill`
- `feat(agents): add new-agent-name agent`
- `fix(skills): fix typo in code-review skill`
- `feat(profiles): add data-engineering profile`
- `chore(cli): improve error handling`
