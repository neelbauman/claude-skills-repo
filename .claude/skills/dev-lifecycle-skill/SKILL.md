---
name: dev-lifecycle
description: >
  Doorstop + Gherkin + Spec-Weaver で仕様管理された開発ライフサイクル全体（分析→設計→計画→実装→検証・コミット）を統制するスキル。
  実装計画をPLANドキュメントとしてDoorstopに永続化し、拡張ドキュメント階層（DESIGN/PLAN/ADR/RESEARCH）を管理する。
  ユーザーが要件や仕様を伝えた場合、および開発に取り組む場合は必ずこのスキルを使うこと。
  ユーザーが「開発」「実装」「設計」「ADR」「仕様」「要件」を話題にした場合もこのスキルをつかうこと。
---

# 開発ライフサイクル管理スキル

## 絶対ルール

1. **MUST**: フェーズを飛ばしてはならない。Phase 1 → 2 → 3 → 4 → 5 → 6 の順に進むこと。
2. **MUST**: 各フェーズ開始時に `## 🔷 Phase N: 名称` を出力すること。
3. **MUST**: `⛔ STOP` マーカーではユーザー承認を待つこと。承認なしに次フェーズへ進んではならない。
4. **MUST**: doorstop / spec-weaver コマンドを実行すること（省略禁止）。
5. **MUST NOT**: Phase 2 を経ずにコードを書き始めてはならない（小規模バグ修正を除く）。

---

## フェーズ判定

タスクを受け取ったら、まず開始フェーズを判定すること。

| タスクの種類 | 開始フェーズ |
|---|---|
| 新機能 / 大きな変更 | **Phase 1** から開始 |
| 小さなバグ修正（1〜2ファイル） | **Phase 4** から開始（Phase 5・6 は必須） |
| 仕様の質問のみ | `spec-weaver trace` で調査して回答（フェーズ不要） |

---

## Phase 1: 分析

出力: `## 🔷 Phase 1: 分析`

### 必須ステップ

1. ユーザーの要求を正確に理解すること。不明点があれば質問すること。
2. 以下のコマンドを実行し、関連する既存仕様を特定すること:
   ```bash
   uv run spec-weaver status
   uv run spec-weaver trace <関連ID> -f ./specification/features
   ```
3. Grep / Glob / Read ツールで影響を受けるコードモジュールを特定すること。
4. 分析結果を以下の形式でユーザーに報告すること:
   - 関連する REQ / SPEC のリスト
   - 影響を受けるモジュール
   - 新規 REQ / SPEC / `.feature` の追加が必要か
   - リスク・懸念事項

### **⛔ STOP: 分析結果を報告し、ユーザーの承認を得ること。承認なしに Phase 2 へ進んではならない。**

---

## Phase 2: 設計

出力: `## 🔷 Phase 2: 設計`

### 必須ステップ

1. 新規/更新が必要な REQ / SPEC を **doorstop-gherkin-skill の手順で** 作成・更新すること:
   ```bash
   doorstop add REQ          # 新規REQ追加
   doorstop add SPEC         # 新規SPEC追加
   doorstop link SPEC-xxx REQ-xxx   # リンク設定
   ```
2. REQ / SPEC の YAML に `status: draft` を設定すること。
3. 規模に応じて設計ドキュメントを作成すること（判定表を参照）:

   | 規模 | DESIGN | ADR | PLAN |
   |---|---|---|---|
   | 大規模（複数モジュール横断） | **必須** | 必要に応じて | **必須** |
   | 中規模（単一モジュール内の大きな変更） | 推奨 | — | 推奨 |
   | 小規模（数ファイルの変更） | — | — | 任意 |

4. `.feature` ファイルの追加・更新方針を決定し、**bdd-behave-expert-skill の Gherkin 規則に従って** Feature / Scenario を記述すること。
5. 設計方針をユーザーに提示すること。

### **⛔ STOP: 設計方針を提示し、ユーザーの承認を得ること。承認なしに Phase 3 へ進んではならない。**

---

## Phase 3: 計画

Plan Modeで立てた実装計画などをドキュメントとして保存する。

出力: `## 🔷 Phase 3: 計画`

### 必須ステップ

1. 実装タスクを具体的なステップに分解すること。各ステップは以下を満たすこと:
   - 1つの論理的な変更単位である
   - 完了条件が明確である
   - 依存関係が明示されている
2. 中〜大規模の場合、PLAN ドキュメントを Doorstop に作成すること:
   ```bash
   doorstop add PLAN
   doorstop link PLAN-xxx SPEC-xxx
   ```
3. タスクの実行順序を整理すること（依存関係順 → リスク順 → テスト容易性）。
4. 実装計画をユーザーに提示すること。

### **⛔ STOP: 実装計画を提示し、ユーザーの承認を得ること。承認なしに Phase 4 へ進んではならない。**

---

## Phase 4: 実装

出力: `## 🔷 Phase 4: 実装`

### 必須ステップ

1. 計画に沿ってコードを変更すること。
2. コード変更に伴い仕様が変わる場合は、**コードと同時に** SPEC / `.feature` を更新すること（doorstop-gherkin-skill の手順に従う）。
3. `.feature` ファイルを追加・更新した場合は、**bdd-behave-expert-skill の手順で** ステップ定義を実装すること:
   ```bash
   # scaffold で雛形を生成（必須・手書き禁止）
   uv run spec-weaver scaffold ./specification/features --out-dir features/steps
   # 生成された雛形の NotImplementedError を仕様に従って肉付けする
   ```
4. 関連する REQ / SPEC の `status` を `in-progress` に更新すること。
5. テストを実行し、確認すること:
   ```bash
   uv run pytest tests/ -q
   uv run behave          # BDD シナリオの実行（失敗は仕様と実装の乖離を示すため正常）
   ```
5. 実装したファイルのパスを、対応する SPEC の YAML に `impl_files` カスタム属性としてリスト形式で記述すること:
   ```yaml
   impl_files:
     - src/your_module.py
     - src/another_module.py
   ```
   また、実装ファイルの先頭付近にアノテーションも記述すること:
   ```python
   # implements: SPEC-XXX
   ```
   > **注意**: Doorstop の `ref` フィールドは文字列専用のため、ファイルパスリストには必ず `impl_files` を使うこと。
   >
   > **検証方法**: `uv run spec-weaver audit --check-impl ./specification/features` で impl_files とアノテーションの乖離を確認できる。

### 必須条件（Phase 5 へ進む前に全て満たすこと）

- 計画の全タスクが完了している
- `pytest` テストが通過している
- `.feature` を追加・更新した場合、対応するステップ定義が実装されている（`behave` 実行済み）
- 実装ファイルが対応 SPEC の `impl_files` カスタム属性に記述されている

---

## Phase 5: 波及確認

出力: `## 🔷 Phase 5: 波及確認`

**目的**: 変更されたアイテムを起点に、関連する全てのアイテム（REQ / SPEC / `.feature` / DESIGN / PLAN / コード）が整合しているか精査し、必要な更新を行う。

### 必須ステップ

1. 今回変更した全アイテム（REQ / SPEC / `.feature` / DESIGN / PLAN / コード）をリストアップすること。
2. 変更した各アイテムについて、`spec-weaver trace` で上位・下位の関連アイテムを洗い出すこと:
   ```bash
   uv run spec-weaver trace <変更したID> -f ./specification/features
   ```
3. 以下の観点で各関連アイテムを精査し、不整合があれば更新すること:

   | 変更元 | 精査対象 | 確認すべきこと |
   |---|---|---|
   | REQ を変更 | 子 SPEC / `.feature` / DESIGN | SPEC の記述が REQ の変更を反映しているか |
   | SPEC を変更 | 親 REQ / `.feature` / DESIGN / PLAN | `.feature` のシナリオが SPEC と一致しているか |
   | `.feature` を変更 | 対応 SPEC / テストコード | SPEC の testable 属性やタグが正しいか |
   | DESIGN を変更 | 親 SPEC / 実装コード | コードが設計と一致しているか |
   | コードを変更 | 対応 SPEC / `.feature` | 仕様に記載のない振る舞いが増えていないか |

4. 精査結果と更新内容をユーザーに報告すること。

### **⛔ STOP: 波及確認の結果を報告し、ユーザーの承認を得ること。承認なしに Phase 6 へ進んではならない。**

---

## Phase 6: 検証・コミット

出力: `## 🔷 Phase 6: 検証・コミット`

### 必須コマンド（この順序で実行すること。失敗したら修正して再実行）

```bash
# 1. ユニットテスト
uv run pytest tests/ -q

# 2. BDD 受け入れテスト（失敗は仕様と実装の乖離を示す）
uv run behave

# 3. Spec-Weaver 監査（仕様 ↔ Gherkin の整合性）
uv run spec-weaver audit ./specification/features

# 4. Doorstop バリデーション（リンク整合性）
doorstop

# 5. ドキュメント再生成
uv run spec-weaver build ./specification/features --out-dir .specification
```

### 必須ステップ

1. 上記4つの検証コマンドを **全て** 実行し、通過を確認すること。
2. 関連する REQ / SPEC の `status` を `implemented` に更新すること。
3. ステータスを確認すること:
   ```bash
   uv run spec-weaver status
   ```
4. コミット規約（Conventional Commits）に従ってコミットすること。scope に仕様ID を記載すること:
   ```
   feat(SPEC-xxx): 変更の概要
   ```
   > 詳細は `references/commit-conventions.md` を参照。

---

## 拡張ドキュメント階層

REQ → SPEC の下に以下のドキュメント型を配置できる。

```
REQ（ビジネス要件）
└── SPEC（システム仕様）
    ├── DESIGN（アーキテクチャ・コンポーネント設計）
    ├── PLAN（実装タスク分解・順序）
    ├── ADR（技術選定の記録）
    └── RESEARCH（技術調査・PoC結果）
```

初回セットアップが必要な場合:
```bash
doorstop create DESIGN ./specification/designs --parent SPEC
doorstop create PLAN ./specification/plans --parent SPEC
doorstop create ADR ./specification/adrs --parent SPEC
doorstop create RESEARCH ./specification/research --parent SPEC
```

> 各ドキュメント型の詳細は `references/document-types.md` を参照。

---

## 他スキルとの役割分担

| 操作 | 担当スキル |
|---|---|
| REQ / SPEC の YAML 作成・編集、`.feature` 作成・編集、Spec-Weaver コマンドの使い方 | **doorstop-gherkin-skill** |
| `.feature` の Gherkin 設計、`scaffold` 実行、ステップ定義の実装 | **bdd-behave-expert-skill** |
| フェーズ進行管理、コミット規約、拡張ドキュメント管理 | **本スキル** |

### bdd-behave-expert-skill を呼び出すタイミング

| フェーズ | 操作 |
|---|---|
| **Phase 2 (設計)** | `.feature` ファイルの新規作成・更新時、Gherkin 記述規則を適用する |
| **Phase 4 (実装)** | `.feature` を追加・更新した後、`scaffold` でステップ雛形を生成し、ステップ定義を実装する |

---

## 参照ファイル

| ファイル | 読むタイミング |
|---|---|
| `references/workflow-phases.md` | 各フェーズの詳細手順を確認するとき |
| `references/commit-conventions.md` | コミットメッセージを作成するとき |
| `references/document-types.md` | DESIGN / PLAN / ADR / RESEARCH を作成するとき |
