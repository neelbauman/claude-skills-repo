# Spec-Weaverの使い方

### 1. 仕様とテストの紐付けルール

Doorstopの仕様書（YAML）で発番されたIDを、Gherkin（`.feature`）の **タグ** として記述することで両者を紐付けます。

**Doorstop側 (`specs/SPEC-001.yml`)**

```yaml
active: true
testable: true
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

### 2. 監査（Audit）コマンドの実行

`audit` コマンドを使用して、仕様とテストの乖離をチェックします。

```bash
# 基本的な実行（カレントディレクトリをプロジェクトルートとする場合）
spec-weaver audit ./features

# モノレポ環境などで、Doorstopのルートディレクトリが別にある場合
spec-weaver audit ./backend/tests/features --repo-root ./docs/doorstop

# プレフィックスを変更する場合（デフォルトは "SPEC"）
spec-weaver audit ./features --prefix REQ

```

### 3. 出力例

仕様と実装に乖離がある場合、エラー（終了コード1）と共に詳細な表が出力されます。

```text
❌ テストが実装されていない仕様 (Untested Specs):
Missing Spec ID
---------------
SPEC-002

⚠️ 仕様書に存在しない孤児タグ (Orphaned Tags):
Orphaned Tag
---------------
@SPEC-003

```

乖離がない場合は、成功メッセージ（終了コード0）が出力されます。

### 4. サイトのビルド（`build` コマンド）

ブラウザで閲覧できる仕様ドキュメントサイトを生成します。

```bash
# 基本的な実行
spec-weaver build ./specification/features

# 出力先を指定
spec-weaver build ./specification/features --out-dir .specification

# MkDocsで閲覧
mkdocs serve -f .specification/mkdocs.yml
```

**生成されるコンテンツ:**

| ページ | 内容 |
|---|---|
| `requirements.md` | 要件一覧（カバレッジ割合・兄弟リンク付き） |
| `specifications.md` | 仕様一覧（カバレッジ割合・兄弟リンク付き） |
| `items/REQ-001.md` | 要件詳細（関連仕様・兄弟要件・集計カバレッジ） |
| `items/SPEC-001.md` | 仕様詳細（関連要件・兄弟仕様・シナリオリンク） |
| `features/xxx.md` | `.feature` をブラウザで読めるMarkdownに変換したページ |

**カバレッジ表示の見方:**

- `🟢 1/1 (100%)` — 全シナリオカバー済み
- `🟡 2/4 (50%)` — 一部カバー済み
- `🔴 0/3 (0%)` — 未カバー
- `⚪️ -` — テスト対象外（`testable: false`）

**REQのカバレッジ**: 関連する全テスト対象SPECのうち、Gherkinシナリオが存在するものの割合を集計して表示します。

**兄弟リンク**: 同じ親REQ（またはREQグループ）にリンクされているSPEC同士は「兄弟仕様」として相互リンクされます。

---

## ⚙️ 高度な設定（テスト対象外の仕様）

「UIのカラーコード」や「ライセンス表記」など、Gherkinでの振る舞いテストが不可能な仕様は、DoorstopのYAMLファイルに `testable: false` 属性を追記することで、Spec-Weaverの監査対象から除外できます。

```yaml
# SPEC-005.yml
active: true
testable: false
text: |
  ログインボタンの背景色は青色とすること。

```
