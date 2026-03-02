# Spec-Weaverの使い方

## コマンド一覧

| コマンド | 概要 |
|---|---|
| `audit` | 仕様とGherkinタグの乖離を検知、`--check-impl` で実装ファイルリンクも検証（CI連携向け） |
| `status` | 実装ステータスの一覧・フィルタリング表示 |
| `build` | カバレッジ・テスト結果統合ドキュメントサイトの生成 |
| `trace` | 任意アイテムを起点としたトレーサビリティツリー表示、`--show-impl` で実装ファイルも表示 |
| `scaffold` | `.feature` ファイルから behave テストコードの雛形を生成・差分マージ |
| `review` | `.feature` ファイルまたは Doorstop アイテムをレビュー済み状態にする |
| `clear` | Doorstop YAML の gherkin_fingerprints を更新し Suspect 状態を解除 |
| `semantic-review` | 仕様・Gherkin・実装コードの意味的整合性を Claude でレビュー |

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
  CORE-001

⚠️ 仕様書に存在しない孤児タグ (Orphaned Tags):
  @SPEC-003

⚠️ レビューが必要なSuspect仕様:
  VIS-001  上位要件が変更されました。レビューが必要です。
```

**オプション:**

| オプション | 短縮 | デフォルト | 説明 |
|---|---|---|---|
| `--repo-root` | `-r` | カレント | Doorstopリポジトリのルート |
| `--prefix` | `-p` | なし（全対象） | 監査対象とする仕様IDのプレフィックス |
| `--stale-days` | | `90` | updated_at からの経過日数がこの値を超えたアイテムを stale として警告（0で無効） |
| `--check-impl` | | 無効 | 実装ファイルリンクの検証を有効化 |
| `--extensions` | | 全ファイル | アノテーションスキャン対象の拡張子（カンマ区切り、例: `py,ts`） |

### `--check-impl` オプション — 実装ファイルリンクの検証

`--check-impl` を指定すると、DoorstopのYAML `impl_files` カスタム属性とコードアノテーションの整合性を検証するセクションが追加されます。

```bash
# 実装ファイルリンクの検証を追加する場合
spec-weaver audit ./specification/features --check-impl

# 特定の拡張子のみスキャンする場合
spec-weaver audit ./specification/features --check-impl --extensions py,ts
```

`--check-impl` 付きの出力例：

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🔗 実装ファイルリンクの検証
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
❌ 存在しないファイルへの impl_files:
   SPEC-001 → src/spec_weaver/old_file.py (not found)

⚠️  impl_files のみ（アノテーションなし）:
   CORE-001 → src/spec_weaver/cli.py

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

**オプション:**

| オプション | 短縮 | デフォルト | 説明 |
|---|---|---|---|
| `--repo-root` | `-r` | カレント | Doorstopリポジトリのルート |
| `--feature-dir` | `-f` | `specification/features` | `.feature` ファイルディレクトリ |
| `--filter` | `-F` | なし（全表示） | 表示するステータスで絞り込む |

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

**オプション:**

| オプション | 短縮 | デフォルト | 説明 |
|---|---|---|---|
| `--repo-root` | `-r` | カレント | Doorstopリポジトリのルート |
| `--out-dir` | `-o` | `.specification` | ドキュメント出力先ディレクトリ |
| `--prefix` | `-p` | `SPEC` | Gherkinタグとして主に扱うデフォルトプレフィックス |
| `--test-results` | `-t` | なし | Cucumber互換JSONレポートのパス |

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
spec-weaver trace TRC-003 -f ./specification/features --show-impl

# 特定の拡張子のみスキャンする場合
spec-weaver trace TRC-003 -f ./specification/features --show-impl --extensions py,ts
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
│   ├── CORE-001 データ抽出基盤 ✅ implemented
│   │   └── 🥒 data_extraction.feature
│   │       └── Scenario: データ抽出基盤
│   └── SPEC-003 audit コマンド仕様 ✅ implemented
│       └── 🥒 audit.feature
│           └── Scenario: audit コマンド
└── SPEC-001 コア・アーキテクチャ ✅ implemented
```

**`--show-impl` 付きのツリー出力例（`TRC-003` を起点）:**

```
REQ-012 仕様アイテムと実装ファイルのリンク管理 ✅ implemented
└── ★ TRC-003 コードアノテーションスキャン ✅ implemented
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

## 6. `scaffold` コマンド — behave テストコード雛形の生成

`.feature` ファイルから behave テストコードの雛形を自動生成します。既存のステップ定義ファイルとの差分マージにも対応しています。

```bash
# 基本的な実行
spec-weaver scaffold ./specification/features --out-dir tests/features

# 既存ファイルを全上書きする場合
spec-weaver scaffold ./specification/features --out-dir tests/features --overwrite

# Git未コミット変更の確認をスキップして強制マージ
spec-weaver scaffold ./specification/features --out-dir tests/features --force
```

**オプション:**

| オプション | 短縮 | デフォルト | 説明 |
|---|---|---|---|
| `--out-dir` | `-o` | `tests/features` | テストコード出力先ディレクトリ |
| `--overwrite` | | 無効 | 既存ファイルを全上書きする |
| `--repo-root` | `-r` | カレント | Git dirty チェック用リポジトリルート |
| `--force` | | 無効 | Git 未コミット変更の確認をスキップして強制マージ |

**生成されるファイル:**

- `step_<feature_stem>.py` — 各 `.feature` に対応するステップ定義ファイル
- 関数名はシナリオ名/ステップ文の SHA256 先頭8文字で生成（日本語対応）
- 同一ステップは重複排除される

---

## 7. `review` コマンド — レビュー済みマーク

指定したアイテムをレビュー済み状態にします。対象に応じて動作が変わります。

```bash
# .feature ファイルのレビュー（フィンガープリント計算・書き込み）
spec-weaver review specification/features/audit.feature

# Doorstop アイテムID のレビュー（doorstop review を呼び出し）
spec-weaver review SPEC-003

# .yml ファイルを直接指定してレビュー
spec-weaver review specification/specs/SPEC-003.yml
```

**対象別の動作:**

| 対象 | 動作 |
|---|---|
| `.feature` ファイル | 構造ハッシュ（フィンガープリント）を計算し、ファイル先頭コメントに書き込む |
| Doorstop アイテムID | `doorstop review` コマンドを呼び出してレビュー済みにする |
| `.yml` ファイル | ファイル名からアイテムIDを取得し `doorstop review` を呼び出す |

**オプション:**

| オプション | 短縮 | デフォルト | 説明 |
|---|---|---|---|
| `--feature-dir` | `-f` | `specification/features` | `.feature` ファイルディレクトリ |
| `--repo-root` | `-r` | カレント | Doorstopリポジトリのルート |

> **注意**: review 実行後に自動的に `audit` が実行され、現在の整合性状態が表示されます。

---

## 8. `clear` コマンド — Suspect 状態の解除

Doorstop YAML の `gherkin_fingerprints` を現在の Gherkin ハッシュで更新し、Suspect 状態を解除します。

```bash
# アイテムIDを指定して解除
spec-weaver clear SPEC-003 --feature-dir ./specification/features

# .feature ファイルを指定して、ファイル内の全アイテムを一括解除
spec-weaver clear specification/features/audit.feature --feature-dir ./specification/features
```

**前提条件:**
- 対象アイテムが**レビュー済み**であること（未レビューの場合は先に `review` が必要）
- 上位アイテムが未レビューの場合（`suspect-with-unreviewed` 状態）も clear 不可

**オプション:**

| オプション | 短縮 | デフォルト | 説明 |
|---|---|---|---|
| `--feature-dir` | `-f` | `specification/features` | `.feature` ファイルディレクトリ |
| `--repo-root` | `-r` | カレント | Doorstopリポジトリのルート |

### review / clear のワークフロー

仕様変更後の典型的なワークフロー：

```
1. 仕様変更 → .feature ファイルを修正
2. spec-weaver review <feature_file>   → フィンガープリント更新
3. spec-weaver clear <feature_file>    → Suspect 状態を解除
4. spec-weaver audit <feature_dir>     → 整合性確認
```

---

## 9. `semantic-review` コマンド — Claude による意味的レビュー

仕様書（YAML）、Gherkin（`.feature`）、実装コードの意味的整合性を Claude で自動レビューします。

```bash
# 特定のアイテムをレビュー
spec-weaver semantic-review --item SPEC-003

# 全仕様アイテムを並列レビュー
spec-weaver semantic-review --all

# JSON形式で出力
spec-weaver semantic-review --item SPEC-003 --output json

# 重大度フィルタリング（medium以上のみ表示）
spec-weaver semantic-review --item SPEC-003 --min-severity medium

# high以上のfindingがあれば終了コード1を返す（CI連携）
spec-weaver semantic-review --all --fail-on high
```

**オプション:**

| オプション | 短縮 | デフォルト | 説明 |
|---|---|---|---|
| `--item` | `-i` | なし | レビュー対象のアイテムID（`--all` と排他） |
| `--all` | | 無効 | 全仕様アイテムを並列レビュー（`--item` と排他） |
| `--feature-dir` | `-f` | `specification/features` | `.feature` ファイル検索ディレクトリ |
| `--repo-root` | `-r` | カレント | Doorstopリポジトリのルート |
| `--output` | `-o` | `text` | 出力形式: `text`（Markdown） / `json` |
| `--min-severity` | | `low` | 表示する finding の最低重大度: `low` / `medium` / `high` |
| `--fail-on` | | なし | 指定重大度以上の finding があれば終了コード 1 を返す |
| `--max-workers` | | `3` | `--all` 時の並列 Claude プロセス数 |
| `--timeout` | | `300` | Claude プロセスの最大待機秒数 |

> **注意**: このコマンドの実行には Claude API アクセスが必要です。

---

## 10. 実装ファイルとのリンク管理（`impl_files` + アノテーション）

仕様アイテムと実装ファイルを双方向でリンクする仕組みです。2つのアプローチを組み合わせて使います。

### アプローチ1: `impl_files` カスタム属性（YAML側）

DoorstopのYAMLに `impl_files` カスタム属性を追加し、実装ファイルパスのリストを記録します。

```yaml
# TRC-003.yml
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
# implements: TRC-003
# implements: TRC-003, QA-003   # 複数IDをカンマ区切りで列挙可能
```

```typescript
// implements: TRC-003
```

```sql
-- implements: TRC-003
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

## 高度な設定

### テスト対象外の仕様

Gherkinでの振る舞いテストが不可能な仕様は `testable: false` を追記して監査対象から除外します。

```yaml
# QA-001.yml
active: true
testable: false
text: |
  ログインボタンの背景色は青色とすること。
```

### 実装ステータスの更新手順

1. 対象YAMLファイルを開き `status: <値>` を追記・更新する
2. `spec-weaver status --filter <値>` で変更を確認する
3. 必要に応じて `spec-weaver build` でドキュメントを再生成する
