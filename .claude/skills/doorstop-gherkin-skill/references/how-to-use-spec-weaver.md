# Spec-Weaverの使い方

## コマンド一覧

| コマンド | 概要 |
|---|---|
| `audit` | 仕様とGherkinタグの乖離を検知、`--check-impl` で実装ファイルリンクも検証（CI連携向け） |
| `status` | 実装ステータスの一覧・フィルタリング表示 |
| `build` | カバレッジ・テスト結果統合ドキュメントサイトの生成 |
| `trace` | 任意アイテムを起点としたトレーサビリティツリー表示、`--show-impl` で実装ファイルも表示 |

---

## 1. 仕様とテストの紐付けルール

Doorstopの仕様書（YAML）で発番されたIDを、Gherkin（`.feature`）の **タグ** として記述することで両者を紐付けます。

**Doorstop側 (`specs/SPEC-001.yml`)**

```yaml
active: true
testable: true
links:
- REQ-001
text: |
  パスワードはハッシュ化して保存すること。
```

**Gherkin側 (`features/login.feature`)**

```gherkin
@SPEC-001
Feature: ユーザー認証
  Scenario: 正しいパスワードでのログイン
    Given ...
```

---

## 2. `audit` コマンド — 仕様とテストの乖離検知

```bash
# 基本的な実行（カレントディレクトリをプロジェクトルートとする場合）
spec-weaver audit ./features

# モノレポ環境などでDoorstopのルートが別にある場合
spec-weaver audit ./backend/tests/features --repo-root ./docs/doorstop

# プレフィックスを限定する場合（デフォルトは全プレフィックス）
spec-weaver audit ./features --prefix REQ
```

**終了コード**: 0（乖離なし）/ 1（乖離あり、またはSuspect検出）

乖離がある場合の出力例：

```
❌ テストが実装されていない仕様 (Untested Specs):
  SPEC-002

⚠️ 仕様書に存在しない孤児タグ (Orphaned Tags):
  @SPEC-003

⚠️ レビューが必要なSuspect仕様:
  SPEC-004  上位要件が変更されました。レビューが必要です。
```

### `--check-impl` オプション — 実装ファイルリンクの検証

`--check-impl` を指定すると、DoorstopのYAML `impl_files` カスタム属性とコードアノテーションの整合性を検証するセクションが追加されます。

```bash
# 実装ファイルリンクの検証を追加する場合
spec-weaver audit ./specification/features --check-impl

# 特定の拡張子のみスキャンする場合
spec-weaver audit ./specification/features --check-impl --extensions py,ts

# --extensions を省略すると全テキストファイルをスキャン
```

**オプション:**

| オプション | 説明 |
|---|---|
| `--check-impl` | 実装ファイルリンクの検証を有効化（デフォルト: 無効） |
| `--extensions TEXT` | スキャン対象の拡張子をカンマ区切りで指定（例: `py,ts`）。未指定時は全テキストファイルを対象 |

`--check-impl` 付きの出力例：

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🔗 実装ファイルリンクの検証
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
❌ 存在しないファイルへの impl_files:
   SPEC-001 → src/spec_weaver/old_file.py (not found)

⚠️  impl_files のみ（アノテーションなし）:
   SPEC-002 → src/spec_weaver/cli.py

⚠️  アノテーションのみ（impl_files なし）:
   SPEC-003 ← src/spec_weaver/gherkin.py

✅ リンク検証 完了
```

3種類の乖離を検出します：
- `❌` — `impl_files` に記載されたパスがリポジトリに存在しない（壊れたリンク）
- `⚠️ impl_files のみ` — YAMLに記載があるがコードアノテーションがない
- `⚠️ アノテーションのみ` — コードアノテーションがあるがYAMLに記載がない

> **注意**: `--check-impl` が指定されない場合、既存の audit 動作は一切変わりません。

---

## 3. `status` コマンド — 実装ステータス管理

DoorstopのYAMLに `status` カスタム属性を追記して実装進捗を管理します。

```yaml
# SPEC-001.yml に追記
status: in-progress
```

利用可能な値: `draft` / `in-progress` / `implemented` / `deprecated`

```bash
# 全アイテムのステータス一覧
spec-weaver status

# 特定ステータスで絞り込み
spec-weaver status --filter in-progress
spec-weaver status --filter draft
```

---

## 4. `build` コマンド — Living Documentationサイト生成

カバレッジ・相互リンク・テスト結果を統合したドキュメントサイトを生成します。

```bash
# 基本的な実行
spec-weaver build ./specification/features --out-dir .specification

# Cucumber互換のテスト実行結果（JSON）を組み込む場合
spec-weaver build ./specification/features --out-dir .specification \
    --test-results test-results.json

# MkDocsでブラウザ表示
mkdocs serve -f .specification/mkdocs.yml
```

**生成されるコンテンツ:**

| ページ | 内容 |
|---|---|
| `<prefix>.md` | 各ドキュメントの一覧表（カバレッジ・実装状況・相互リンク付き） |
| `items/REQ-001.md` | 要件詳細（関連仕様・兄弟要件・集計カバレッジ） |
| `items/SPEC-001.md` | 仕様詳細（関連要件・兄弟仕様・シナリオリンク） |
| `features/xxx.md` | `.feature` をブラウザで読めるMarkdownに変換したページ |

**カバレッジバッジの見方:**

- `🟢 1/1 (100%)` — 全シナリオカバー済み
- `🟡 2/4 (50%)` — 一部カバー済み
- `🔴 0/3 (0%)` — 未カバー
- `⚪️ -` — テスト対象外（`testable: false`）

**`--test-results` オプション（Cucumber JSON）:**

pytest-bdd などが出力する Cucumber 互換 JSON を指定すると、各シナリオに実行結果バッジが付与されます。

```json
[
  {
    "uri": "features/login.feature",
    "elements": [
      {
        "name": "正しいパスワードでのログイン",
        "steps": [{"result": {"status": "passed"}}, ...]
      }
    ]
  }
]
```

バッジ: `✅ PASS` / `❌ FAIL` / `⏭️ SKIP` / `⏳ PENDING`

---

## 5. `trace` コマンド — トレーサビリティツリー表示

任意のアイテム（REQ / SPEC / `.feature` ファイル名）を起点として、上位要件〜下位仕様〜Gherkinシナリオを階層ツリーで可視化します。

```bash
# REQを起点に全子孫を展開（デフォルト: both = 上位+下位）
spec-weaver trace REQ-001 -f ./specification/features

# SPECを起点に上位REQ + 下位シナリオを表示（both）
spec-weaver trace SPEC-003 -f ./specification/features

# .featureファイルを起点に紐づくSPEC/REQを遡る（up）
spec-weaver trace audit.feature -f ./specification/features --direction up

# 方向の指定
spec-weaver trace REQ-001 --direction down   # 下位のみ（Gherkin含む）
spec-weaver trace SPEC-003 --direction up    # 上位のみ（シナリオなし）

# テーブル形式で出力
spec-weaver trace REQ-001 -f ./specification/features --format flat

# 実装ファイルもツリーに表示（impl_files + アノテーション）
spec-weaver trace SPEC-018 -f ./specification/features --show-impl

# 特定の拡張子のみスキャンする場合
spec-weaver trace SPEC-018 -f ./specification/features --show-impl --extensions py,ts
```

**オプション:**

| オプション | 短縮 | デフォルト | 説明 |
|---|---|---|---|
| `--feature-dir` | `-f` | なし | `.feature` ファイルディレクトリ（Gherkin表示に必要） |
| `--repo-root` | `-r` | カレント | Doorstopリポジトリのルート |
| `--direction` | `-d` | `both` | `up` / `down` / `both` |
| `--format` | | `tree` | `tree`（Rich Tree）/ `flat`（テーブル） |
| `--show-impl` | | 無効 | 実装ファイルをツリーに表示する |
| `--extensions` | | 全ファイル | `--show-impl` 時のアノテーションスキャン対象拡張子（カンマ区切り、例: `py,ts`） |

**ツリー出力例（`SPEC-003` を起点、`both`）:**

```
REQ-001 仕様と実装のトレーサビリティ保証 ✅ implemented
└── REQ-002 監査による品質の継続的担保 ✅ implemented
    └── ★ SPEC-003 audit コマンド仕様 ✅ implemented
        └── 🥒 audit.feature
            └── Scenario: audit コマンド
```

**ツリー出力例（`REQ-001` を起点、`down`）:**

```
★ REQ-001 仕様と実装のトレーサビリティ保証 ✅ implemented
├── REQ-002 監査による品質の継続的担保 ✅ implemented
│   ├── SPEC-002 データ抽出基盤 ✅ implemented
│   │   └── 🥒 data_extraction.feature
│   │       └── Scenario: データ抽出基盤
│   └── SPEC-003 audit コマンド仕様 ✅ implemented
│       └── 🥒 audit.feature
│           └── Scenario: audit コマンド
└── SPEC-001 コア・アーキテクチャ ✅ implemented
```

**`--show-impl` 付きのツリー出力例（`SPEC-018` を起点）:**

```
REQ-012 仕様アイテムと実装ファイルのリンク管理 ✅ implemented
└── ★ SPEC-018 コードアノテーションスキャン ✅ implemented
    ├── 🥒 impl_link.feature
    │   └── Scenario: アノテーションのスキャン
    ├── 📁 src/spec_weaver/impl_scanner.py
    └── 📝 src/spec_weaver/cli.py
```

**表示記号:**

| 記号 | 意味 |
|---|---|
| `★` | 探索起点のアイテム（黄色太字で強調表示） |
| `🥒` | 関連するGherkin `.feature` ファイル |
| `Scenario: ...` | `.feature` 内のシナリオ名 |
| `📁` | `impl_files` 属性由来の実装ファイル（`--show-impl` 時のみ表示） |
| `📝` | アノテーション由来の実装ファイル（`impl_files` に未記載）（`--show-impl` 時のみ表示） |
| `❌` | 存在しないファイルへのリンク（`--show-impl` 時のみ表示） |

**起点の種類:**

- `REQ-001` — Doorstop IDで直接指定
- `SPEC-003` — Doorstop IDで直接指定
- `audit.feature` — ファイル名で指定（内部でタグを逆引きしてSPEC IDを解決）

---

---

## 6. 実装ファイルとのリンク管理（`impl_files` + アノテーション）

仕様アイテムと実装ファイルを双方向でリンクする仕組みです。2つのアプローチを組み合わせて使います。

### アプローチ1: `impl_files` カスタム属性（YAML側）

DoorstopのYAMLに `impl_files` カスタム属性を追加し、実装ファイルパスのリストを記録します。

```yaml
# SPEC-018.yml
active: true
status: implemented
impl_files:
- src/spec_weaver/impl_scanner.py
- src/spec_weaver/doorstop.py
ref: ''
text: |
  （仕様本文）
```

> **注意**: DoorstopのビルトインのYAML `ref` フィールドは**文字列専用**（内部で `.strip()` を呼ぶ）のため、
> リスト形式で格納するとエラーになります。必ず独自のカスタム属性 `impl_files` を使うこと。

- パスはリポジトリルートからの相対パスで記述します
- `impl_files` フィールドが省略された既存アイテムは「リンクなし」として扱い、エラーにしません
- 単一ファイルの場合もリスト形式で記述します

### アプローチ2: コードアノテーション（実装ファイル側）

実装ファイルの先頭付近に `# implements:` アノテーションを記述します。

```python
# implements: SPEC-018
# implements: SPEC-018, SPEC-019   # 複数IDをカンマ区切りで列挙可能
```

```typescript
// implements: SPEC-018
```

```sql
-- implements: SPEC-018
```

- `#`, `//`, `--` のコメント記号をサポートします
- 1行に複数のIDをカンマ区切りで記述できます
- アノテーションのないファイルはエラーにならず、警告なしとして扱われます

### 集約ルール

| 状態 | `audit --check-impl` の結果 | `trace --show-impl` の表示 |
|---|---|---|
| `impl_files` のみ（アノテーションなし） | ⚠️ 警告 | `📁` アイコンで表示 |
| アノテーションのみ（`impl_files` なし） | ⚠️ 警告 | `📝` アイコンで表示 |
| 両方あり | 整合（報告なし） | `📁` アイコンで表示（一度だけ） |
| ファイルが存在しない | ❌ エラー | `❌ <パス> (not found)` で表示 |

### スキャン対象と除外

- デフォルト: リポジトリ内の全テキストファイル
- `--extensions py,ts` で拡張子フィルタリング可能
- 以下のディレクトリは除外: `.git`, `__pycache__`, `.venv`, `venv`, `node_modules`, `.mypy_cache`, `.ruff_cache`

---

## ⚙️ 高度な設定

### テスト対象外の仕様

Gherkinでの振る舞いテストが不可能な仕様は `testable: false` を追記して監査対象から除外します。

```yaml
# SPEC-005.yml
active: true
testable: false
text: |
  ログインボタンの背景色は青色とすること。
```

### 実装ステータスの更新手順

1. 対象YAMLファイルを開き `status: <値>` を追記・更新する
2. `spec-weaver status --filter <値>` で変更を確認する
3. 必要に応じて `spec-weaver build` でドキュメントを再生成する
