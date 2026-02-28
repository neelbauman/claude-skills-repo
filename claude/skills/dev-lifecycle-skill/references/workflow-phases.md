# ワークフローフェーズ 詳細リファレンス

各フェーズの詳細な手順とテンプレートを記載する。
SKILL.md の各 Phase の補足資料として参照すること。

---

## Phase 1: 分析（SKILL.md Phase 1 に対応）

### 目的

変更の影響範囲を把握し、既存の仕様体系との関係を明確にする。
**このフェーズを飛ばすと、後続フェーズで手戻りが発生するリスクが高まる。**

### 手順

#### 1.1 要求の理解

ユーザーの要求を正確に把握する。不明点があれば確認する。

- 何を実現したいのか（ゴール）
- なぜ必要なのか（動機・背景）
- 制約条件はあるか（技術的制約、時間制約）

#### 1.2 既存仕様のスキャン

```bash
# 関連しそうなキーワードで仕様を検索
# （Grep ツールで specification/ 配下を検索）

# 関連するアイテムのトレーサビリティを確認
uv run spec-weaver trace <関連ID> -f ./specification/features

# 全体のステータスを確認
uv run spec-weaver status
```

#### 1.3 コードベースの調査

```bash
# 影響を受けるモジュールを特定
# （Grep / Glob / Read ツールを使用）

# テストの状態を確認
uv run pytest tests/ -q --collect-only
```

#### 1.4 分析結果の報告テンプレート

```markdown
## 分析結果

### 関連する既存仕様
- REQ-xxx: （概要）
- SPEC-xxx: （概要）

### 影響を受けるモジュール
- `src/xxx/yyy.py`: （影響内容）

### 必要な仕様変更
- 新規 REQ の追加: （内容）
- 新規 SPEC の追加: （内容）
- 既存 SPEC-xxx の更新: （変更内容）
- 新規 `.feature` の追加: （内容）

### リスク・懸念事項
- （あれば記載）
```

### ⛔ STOP: 分析結果を報告し、ユーザーの承認を得ること。

---

## Phase 2: 設計（SKILL.md Phase 2 に対応）

### 目的

実装方針を決定し、必要に応じて設計ドキュメント（DESIGN / ADR / RESEARCH）を作成する。

### 手順

#### 2.1 仕様の作成・更新

doorstop-gherkin-skill の手順に従い、REQ / SPEC を作成・更新する。

```bash
# 新規REQの追加
doorstop add REQ
doorstop edit REQ-xxx

# 新規SPECの追加とリンク
doorstop add SPEC
doorstop link SPEC-xxx REQ-xxx
doorstop edit SPEC-xxx
```

`status: draft` を設定すること。

#### 2.2 設計ドキュメントの作成

規模判定に基づいて、必要なドキュメントを作成する。

**大規模変更の場合:**

```bash
# DESIGNドキュメント（アーキテクチャ設計）
doorstop add DESIGN
doorstop link DESIGN-xxx SPEC-xxx

# ADR（技術選定がある場合）
doorstop add ADR
doorstop link ADR-xxx SPEC-xxx

# RESEARCH（調査が必要な場合）
doorstop add RESEARCH
doorstop link RESEARCH-xxx SPEC-xxx
```

各ドキュメント型のテンプレートは `document-types.md` を参照。

#### 2.3 `.feature` ファイルの方針決定と記述

- 新しい振る舞いには新しいシナリオが必要か
- 既存のシナリオを更新する必要があるか
- テスト不可能な仕様には `testable: false` を設定する

`.feature` を実際に記述する場合は **bdd-behave-expert-skill の Gherkin 規則** に従うこと。

主なルール（詳細は bdd-behave-expert-skill を参照）:
- Feature タグに `@SPEC-xxx` を付与して Doorstop とリンクする
- Rule キーワードでビジネスルールを明示する
- 共通前提は `Background` に集約する
- 複数データは `Scenario Outline + Examples` または `Data Table` で表現する
- UI・API・DB の実装詳細を Gherkin に書かない（ドメイン用語を使う）

#### 2.4 設計方針の提示

設計内容をユーザーに提示し、承認を得る。

### 設計ドキュメントの粒度ガイド

| ドキュメント型 | いつ作るか | 最低限含めるべき内容 |
|---|---|---|
| DESIGN | 2つ以上のモジュールに影響する場合 | コンポーネント図、インターフェース定義 |
| ADR | 複数の技術的選択肢がある場合 | 選択肢の比較、決定理由 |
| RESEARCH | 未知の技術を使う場合 | 調査結果、PoC結果 |

### ⛔ STOP: 設計方針を提示し、ユーザーの承認を得ること。

---

## Phase 3: 計画（SKILL.md Phase 3 に対応）

### 目的

実装タスクを分解し、実行順序を明確にする。

### 手順

#### 3.1 タスクの分解

実装作業を具体的なステップに分解する。各ステップは以下を満たすこと:

- 1つの論理的な変更単位である
- 完了条件が明確である
- 依存関係が明示されている

#### 3.2 PLAN ドキュメントの作成（中〜大規模の場合）

```bash
doorstop add PLAN
doorstop link PLAN-xxx SPEC-xxx
doorstop edit PLAN-xxx
```

PLAN の `text` フィールドに実装タスクを記述する（テンプレートは `document-types.md` 参照）。

#### 3.3 タスクの順序整理

以下の原則でタスクを並べる:

1. **依存関係順**: 他のタスクが依存するものを先に
2. **リスク順**: 不確実性の高いものを先に（早期にリスクを検出）
3. **テスト容易性**: テストしやすい単位で区切る

#### 3.4 計画の提示

計画をユーザーに提示し、承認を得る。

### ⛔ STOP: 実装計画を提示し、ユーザーの承認を得ること。

---

## Phase 4: 実装（SKILL.md Phase 4 に対応）

### 目的

計画に沿ってコードを変更し、仕様との同期を保つ。

### 手順

#### 4.1 コードの変更

計画の各ステップに沿って実装を進める。

#### 4.2 仕様の同期

コード変更に伴い仕様が変わる場合は、doorstop-gherkin-skill の手順に従って同時に更新する:

- SPEC の `text` を更新する
- 新しい振る舞いがあれば `.feature` にシナリオを追加する

> 注: `created_at` / `updated_at` は Git コミット履歴から自動算出されるため、手動更新は不要。

#### 4.3 ステータスの更新

```bash
# 実装中の SPEC の status を in-progress に変更
# YAML を直接編集するか doorstop edit を使用
```

#### 4.4 BDD ステップ定義の実装（`.feature` を追加・更新した場合）

**bdd-behave-expert-skill の手順に従うこと。** 手順の概要:

1. `scaffold` で雛形を生成する（手書きでゼロから書き始めてはならない）:
   ```bash
   uv run spec-weaver scaffold ./specification/features --out-dir features/steps
   ```
2. 生成された `raise NotImplementedError(...)` を仕様に従って肉付けする:
   - Step は「委譲のみ」（`if` / `for` / ビジネスロジック禁止）
   - 型付きパラメータ（`{count:d}` 等）を使う
   - コメント・`# type: ignore`・docstring は削除しない
3. `behave` を実行してシナリオを確認する（失敗は仕様と実装の乖離を示す）

#### 4.5 テストの実行

```bash
uv run pytest tests/ -q

# .feature を追加・更新した場合は behave も実行する
uv run behave
```

> **注**: BDD テストが失敗することは設計上正常。仕様と実装の乖離を可視化するものであり、
> テストを実装に合わせて歪めてはならない。

---

## Phase 5: 波及確認（SKILL.md Phase 5 に対応）

### 目的

変更されたアイテムを起点に、関連する全てのアイテムが整合しているか精査する。
実装中に見落とした不整合をこのフェーズで検出・修正することで、Phase 6 の検証をスムーズに通過させる。

### 手順

#### 5.1 変更アイテムのリストアップ

今回のタスクで変更・追加した全アイテムを列挙する:

- REQ（新規追加 / 更新したもの）
- SPEC（新規追加 / 更新したもの）
- `.feature` ファイル（新規追加 / 更新したもの）
- DESIGN / PLAN / ADR / RESEARCH（該当するもの）
- コードファイル（変更したモジュール）

#### 5.2 トレーサビリティの走査

変更した各アイテムについて、上位・下位の関連アイテムを `spec-weaver trace` で確認する:

```bash
# 変更した各IDについて実行
uv run spec-weaver trace <変更したID> -f ./specification/features

# 全体のステータス確認
uv run spec-weaver status
```

#### 5.3 精査マトリクス

以下の観点で各関連アイテムを精査し、不整合があれば更新すること:

| 変更元 | 精査対象 | 確認すべきこと |
|---|---|---|
| REQ を変更 | 子 SPEC、`.feature`、DESIGN | SPEC の記述が REQ の変更を反映しているか。受け入れ条件が `.feature` と一致するか |
| SPEC を変更 | 親 REQ、`.feature`、DESIGN、PLAN | シナリオが SPEC と一致しているか。DESIGN の設計方針と矛盾しないか |
| `.feature` を変更 | 対応 SPEC、テストコード | SPEC の `testable` 属性・タグが正しいか。テストコードがシナリオを網羅しているか |
| DESIGN を変更 | 親 SPEC、実装コード | コードが設計と一致しているか。SPEC の記述と矛盾しないか |
| PLAN を変更 | 親 SPEC、実装コード | 計画と実装結果が一致しているか |
| コードを変更 | 対応 SPEC、`.feature` | 仕様に記載のない振る舞いが増えていないか。既存シナリオが壊れていないか |

#### 5.4 精査結果の報告

精査の結果を以下の形式でユーザーに報告する:

```markdown
## 波及確認結果

### 変更アイテム一覧
- （変更したアイテムのリスト）

### 精査結果
- ✅ SPEC-xxx ↔ REQ-xxx: 整合OK
- ⚠️ SPEC-yyy → feature: シナリオの更新が必要 → 更新済み
- ✅ DESIGN-xxx ↔ コード: 整合OK

### 追加で更新したアイテム
- （波及確認により追加更新したもの、なければ「なし」）
```

### ⛔ STOP: 波及確認の結果を報告し、ユーザーの承認を得ること。

---

## Phase 6: 検証・コミット（SKILL.md Phase 6 に対応）

### 目的

全ての整合性チェックを通過させ、変更をコミットする。

### 必須コマンド（この順序で実行すること）

前の検証が失敗した場合は修正してから次に進むこと。

```bash
# Step 1: ユニットテスト
uv run pytest tests/ -q

# Step 2: BDD 受け入れテスト（失敗は仕様と実装の乖離を示す）
uv run behave

# Step 3: Spec-Weaver 監査
uv run spec-weaver audit ./specification/features

# Step 4: Doorstop バリデーション
doorstop

# Step 5: ドキュメント再生成
uv run spec-weaver build ./specification/features --out-dir .specification
```

### ステータスの最終更新

```bash
# 完了した SPEC の status を implemented に変更

# 確認
uv run spec-weaver status
```

> 注: `created_at` / `updated_at` は Git コミット履歴から自動算出されるため、手動更新は不要。

### コミット

コミット規約（`commit-conventions.md`）に従ってコミットする。

```bash
git add <変更ファイル>
git commit -m "feat(SPEC-xxx): 変更の概要"
```

### 検証失敗時の対応

| 検証 | よくある失敗原因 | 対処法 |
|---|---|---|
| pytest | テストロジックの不備 | テストコードを修正して再実行 |
| behave | ステップ未定義 / 仕様と実装の乖離 | `spec-weaver scaffold` を再実行してステップ雛形を生成し、実装する（テストを弱めてはならない） |
| spec-weaver audit | タグの不一致、未テスト仕様 | `.feature` のタグを修正、または `testable: false` を設定 |
| doorstop | リンク切れ、未レビュー | `doorstop link` でリンク修正、`doorstop review all` |

---

## 承認フローの詳細

### いつ承認を求めるか

| タイミング | 必須/任意 | 内容 |
|---|---|---|
| Phase 1 終了時 | **必須** | 分析結果と方向性の確認 |
| Phase 2 終了時 | **必須** | 設計方針の承認 |
| Phase 3 終了時 | **必須** | 実装計画の承認 |
| Phase 4 中 | 任意 | 想定外の問題が発生した場合 |
| Phase 5 終了時 | **必須** | 波及確認の結果と追加更新内容の承認 |
| Phase 6 コミット前 | 任意 | コミット内容の最終確認 |

### 承認の方法

ユーザーに対して明確に「承認を求めている」ことを伝え、
以下のいずれかの応答を待つ:

- **承認**: 次のフェーズに進む
- **修正依頼**: 指摘に基づいて修正し、再度承認を求める
- **差し戻し**: 前のフェーズに戻る
