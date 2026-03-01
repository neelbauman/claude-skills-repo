# spec-weaver セマンティックレビュー 機能仕様書

> ステータス: 設計中
> 対象バージョン: spec-weaver（未定）/ claude-skills-repo（semantic-review-skill）

---

## 1. 目的と背景

### 1.1 解決する問題

spec-weaver の既存の `audit` コマンドは、仕様・Gherkin・実装コードの **リンクの有無** を検証する。
しかし「ファイルは存在しリンクも正しいが、**内容が仕様と食い違う**」状況は検出できない。

このギャップを埋めるのがセマンティックレビュー機能である。

| 既存機能 (`audit`) | 新機能 (`review`) |
|---|---|
| ファイルの存在確認 | ファイルの**内容**を解析 |
| リンクの整合性検証 | 仕様と実装の**意味的一致**を検証 |
| 静的チェック | LLMによる推論 |

### 1.2 位置づけ

```
spec-weaver audit   →  構造・リンクの健全性  （前提条件チェック）
spec-weaver review  →  仕様と実装の意味的整合（セマンティックレビュー）← 本機能
```

---

## 2. アーキテクチャ設計

### 2.1 全体構成

```
┌──────────────────────────────────────────────────────────────────┐
│  spec-weaver review (Pythonプロセス)                              │
│                                                                   │
│  ① spec-weaver trace で仕様に紐づくファイルパスを収集             │
│  ② 収集したファイルセットをタスクとしてキューに積む               │
│  ③ concurrent.futures で claude CLI を N 並列起動                 │
│  ④ 各 claude プロセスが JSON を stdout に出力                     │
│  ⑤ JSON を回収・集約し、最終レポートとして出力                    │
└──────────────────────────────────────────────────────────────────┘
              │  (subprocess)
              ▼  claude -p "<prompt>" --file <spec.yml> --file <impl.py> ...
┌──────────────────────────────────────────────────────────────────┐
│  Claude Code プロセス（独立コンテキスト）                          │
│                                                                   │
│  semantic-review スキルが発動                                      │
│  → 3つの観点でファイル群を評価                                     │
│  → JSON を stdout に出力して終了                                   │
└──────────────────────────────────────────────────────────────────┘
```

### 2.2 設計上の原則

**コンテキスト分離（Context Isolation）**

1仕様アイテム（REQ等）ごとに独立した claude プロセスを起動する。
これにより以下を防ぐ：
- 大規模コードベースでのコンテキスト肥大化（Context Rot）
- 複数仕様の情報が混線することによるハルシネーション

**精度優先のファイル特定**

レビュー対象ファイルは `spec-weaver trace` で取得できる `impl_files` と
`# implements: <ITEM_ID>` アノテーション付きファイルを基本とする。
これらで十分なコンテキストが得られない場合のみ、ディレクトリ探索に拡大する。

---

## 3. 成果物 A: `spec-weaver review` CLIコマンド

### 3.1 コマンドインターフェース

#### 個別レビューモード（開発・デバッグ用）

```bash
spec-weaver review --item <ITEM_ID> [options]
```

例:
```bash
spec-weaver review --item REQ-001
spec-weaver review --item SPEC-042 --feature-dir ./specification/features
spec-weaver review --item REQ-001 --output json > review_req001.json
```

#### 全体並列レビューモード（CI/CD・全体チェック用）

```bash
spec-weaver review --all [options]
```

例:
```bash
spec-weaver review --all
spec-weaver review --all --max-workers 4
spec-weaver review --all --output json > review_all.json
```

### 3.2 オプション一覧

| オプション | デフォルト | 説明 |
|---|---|---|
| `--item <ID>` | — | 単一の仕様アイテムIDを指定（`--all` と排他） |
| `--all` | — | 全仕様アイテムを並列レビュー（`--item` と排他） |
| `--feature-dir <path>` | `./specification/features` | .feature ファイルの検索ディレクトリ |
| `--max-workers <N>` | `3` | `--all` 時の並列Claude プロセス数 |
| `--output <format>` | `text` | 出力形式: `text`（Markdown）/ `json` |
| `--min-severity <level>` | `low` | 指定レベル以上のfinding のみ表示: `low` / `medium` / `high` |
| `--fail-on <level>` | なし | 指定レベル以上のfinding があれば終了コード1（CI用） |

### 3.3 内部処理フロー

#### 個別レビューモード

```
1. spec-weaver trace <ITEM_ID> --feature-dir <dir> --show-impl
   → 関連ファイルパスのリストを取得
     - 仕様YAML (Doorstop)
     - .feature ファイル
     - ステップ定義ファイル
     - プロダクトコード (impl_files / # implements: アノテーション)

2. プロンプトを構築（§3.4 参照）

3. claude -p "<prompt>" --file <file1> --file <file2> ... を実行

4. stdout から JSON を読み取り
   - 非JSON行はすべて無視（claude の思考ログ等）
   - 最後の JSON ブロックを採用

5. --output に応じて表示
```

#### 全体並列レビューモード

```
1. spec-weaver list-items で全アイテムIDを取得

2. 各アイテムIDに対して個別レビューと同じ手順でタスクを構築

3. concurrent.futures.ThreadPoolExecutor(max_workers=N) で並列実行
   - 各タスクは独立した subprocess として claude を起動

4. 完了したタスクから順次結果を回収

5. 全タスク完了後、結果を集約（§5 参照）

6. --output に応じて集約レポートを表示
   - json: 集約JSONを stdout に出力
   - text: サマリー + finding 一覧を Markdown で表示
```

### 3.4 Claude CLI 呼び出し仕様

spec-weaver が構築するプロンプトのテンプレート：

```
spec-weaverセマンティックレビューを実行してください。

対象アイテム: {ITEM_ID}（{ITEM_TITLE}）

添付ファイルの内容を精査し、仕様と実装の意味的な整合性を評価してください。
結果は必ず純粋なJSONのみで出力してください（マークダウンや説明文は不要）。
JSONスキーマは semantic-review スキルの出力仕様に従ってください。
```

ファイルは `--file` フラグで渡す：
```bash
claude \
  -p "$(cat prompt.txt)" \
  --file spec/REQ/REQ-001.yml \
  --file specification/features/auth.feature \
  --file features/steps/auth_steps.py \
  --file src/auth.py
```

**プロンプト内の「spec-weaverセマンティックレビュー」というキーワードが Claude Code の semantic-review スキルを自動発動させる。**

### 3.5 結果の集約

`--all` 実行時の集約レポート構造：

```json
{
  "schema_version": "1.0",
  "reviewed_at": "2025-06-01T12:34:56Z",
  "total_items": 15,
  "failed_items": 2,
  "results": [
    { /* 個別レビューのJSONオブジェクト（§5参照） */ },
    { /* ... */ }
  ],
  "aggregate_summary": {
    "total_findings": 8,
    "by_type": {
      "missing_implementation": 3,
      "undocumented_feature": 2,
      "semantic_contradiction": 3
    },
    "by_severity": {
      "high": 2,
      "medium": 4,
      "low": 2
    }
  }
}
```

---

## 4. 成果物 B: Claude Code スキル（semantic-review-skill）

### 4.1 スキルの役割

spec-weaver が起動した Claude Code プロセスの中で発動し、渡されたファイル群を
3つの評価観点で解析して、結果を JSON として出力する **評価エージェント**。

ユーザーが Claude Code に直接「REQ-001のセマンティックレビューをして」と依頼する
インタラクティブな使い方でも発動する。

### 4.2 トリガーキーワード

以下のキーワードを含む依頼でスキルを発動する：

- `spec-weaverセマンティックレビュー`（spec-weaver CLIからの自動呼び出し）
- `セマンティックレビュー`
- `仕様-コードレビュー` / `仕様とコードのレビュー`
- `仕様と実装の乖離`
- `semantic review`

### 4.3 評価観点（3つ）

#### 観点 1: 実装漏れの検出（Missing Implementation）

**問い**: 仕様書・シナリオに明記されている要件が、実装コードに存在するか？

確認ポイント：
- 仕様YAML の `text` / `rationale` に記述された要件が実装されているか
- Gherkin の `Then` ステップが示す期待動作がコードで実現されているか
- 数値・閾値・制約条件（「5回まで」「10,000円以上」等）がコードに反映されているか
- エラーケースや例外フローが仕様に書かれていれば実装にも存在するか

#### 観点 2: 仕様の欠落・暗黙知の発見（Undocumented Feature）

**問い**: 実装コードに存在する振る舞いが、仕様書から漏れていないか？

確認ポイント：
- エラーハンドリング・例外処理が仕様に記述されているか
- バリデーションロジック（型チェック・範囲チェック等）が仕様に記述されているか
- 実装上の境界値処理・デフォルト値が仕様に記述されているか
- コードコメントに書かれた「暗黙のルール」が仕様に反映されているか

#### 観点 3: 意味的な矛盾・乖離の検出（Semantic Contradiction）

**問い**: 仕様とコードの両方に記述はあるが、内容が食い違っていないか？

確認ポイント：
- 数値・閾値の不一致（仕様:10回、コード:`limit = 5`）
- 条件式の方向性の誤り（仕様:「以上」、コード:`>`）
- 状態遷移の順序・条件の相違
- 仕様の「べき」と実装の「は」の違い（SHOULD vs IS）
- Gherkin の `Given/When/Then` が示すロジックとコードの処理順序の相違

### 4.4 評価プロセス（スキル内の作業手順）

```
Step 1: ファイルの役割を把握する
  - 仕様YAML  → 「要件の源泉」として読む
  - .feature  → 「受け入れ条件」として読む
  - steps.py  → 「テストと実装の橋渡し」として読む
  - product code → 「実際の動作」として読む

Step 2: 仕様側の要求事項をリストアップする
  - 仕様YAML と .feature から「〜すること」「〜しなければならない」を抽出

Step 3: コード側の実装を確認する
  - 抽出した要求事項が実装に存在するか、内容が一致するかを確認

Step 4: コード側から仕様を逆引きする
  - コードの主要なロジック・ハンドリングを起点に、仕様に記述があるかを確認

Step 5: 結果をJSONとして出力する
  - 呼び出しモードに応じた形式で出力（§4.5 参照）
```

### 4.5 出力モード

| 呼び出し元 | 検知方法 | 出力形式 |
|---|---|---|
| spec-weaver CLI (subprocess) | プロンプトに「純粋なJSONのみで出力」の指示あり | JSON のみ（説明文なし） |
| ユーザーの直接依頼 | 上記指示なし | Markdown形式のレポート |

**インタラクティブモード（Markdown）の出力構造：**

```markdown
## セマンティックレビュー結果: REQ-001（ユーザー認証）

### サマリー
- 総Finding数: 3件（high: 1 / medium: 1 / low: 1）

### Findings

#### [HIGH] bcryptハッシュ化が未実装 (missing_implementation)
- **仕様**: `spec/REQ/REQ-001.yml` — "パスワードはbcryptでハッシュ化すること"
- **実装**: `src/auth.py:42` — `self.password = password`（平文保存）
- **説明**: 仕様が要求するハッシュ化が実装されていない。

...
```

---

## 5. JSON 出力スキーマ

### 5.1 個別レビュー結果

```json
{
  "schema_version": "1.0",
  "item_id": "REQ-001",
  "item_title": "ユーザー認証",
  "reviewed_files": [
    "spec/REQ/REQ-001.yml",
    "specification/features/auth.feature",
    "features/steps/auth_steps.py",
    "src/auth.py"
  ],
  "findings": [
    {
      "type": "missing_implementation",
      "severity": "high",
      "title": "bcryptハッシュ化が未実装",
      "description": "仕様ではbcryptによるパスワードハッシュ化が必須とされているが、実装では平文保存している",
      "spec_ref": {
        "file": "spec/REQ/REQ-001.yml",
        "excerpt": "パスワードはbcryptでハッシュ化すること"
      },
      "code_ref": {
        "file": "src/auth.py",
        "line": 42,
        "excerpt": "self.password = password"
      }
    },
    {
      "type": "undocumented_feature",
      "severity": "medium",
      "title": "アカウントロック機能が仕様に記載なし",
      "description": "5回連続失敗でアカウントをロックする実装があるが、仕様書に記載がない",
      "spec_ref": null,
      "code_ref": {
        "file": "src/auth.py",
        "line": 87,
        "excerpt": "if attempts >= 5: lock_account()"
      }
    },
    {
      "type": "semantic_contradiction",
      "severity": "high",
      "title": "ロック閾値の数値不一致",
      "description": "Gherkinでは3回失敗でロックとシナリオに書かれているが、実装は5回になっている",
      "spec_ref": {
        "file": "specification/features/auth.feature",
        "excerpt": "When ログインを 3 回失敗する"
      },
      "code_ref": {
        "file": "src/auth.py",
        "line": 87,
        "excerpt": "if attempts >= 5: lock_account()"
      }
    }
  ],
  "summary": {
    "total_findings": 3,
    "by_type": {
      "missing_implementation": 1,
      "undocumented_feature": 1,
      "semantic_contradiction": 1
    },
    "by_severity": {
      "high": 2,
      "medium": 1,
      "low": 0
    }
  }
}
```

### 5.2 フィールド定義

#### finding オブジェクト

| フィールド | 型 | 必須 | 説明 |
|---|---|---|---|
| `type` | string | ✓ | `missing_implementation` / `undocumented_feature` / `semantic_contradiction` |
| `severity` | string | ✓ | `high` / `medium` / `low` |
| `title` | string | ✓ | finding の短い説明（1行） |
| `description` | string | ✓ | finding の詳細説明 |
| `spec_ref` | object\|null | ✓ | 仕様側の根拠（存在しない場合は null） |
| `code_ref` | object\|null | ✓ | コード側の根拠（存在しない場合は null） |

#### ref オブジェクト（spec_ref / code_ref）

| フィールド | 型 | 必須 | 説明 |
|---|---|---|---|
| `file` | string | ✓ | ファイルパス（プロジェクトルートからの相対パス） |
| `line` | integer | — | 該当行番号（特定できる場合） |
| `excerpt` | string | ✓ | 該当箇所の引用テキスト |

### 5.3 severity の判断基準

| severity | 基準 |
|---|---|
| `high` | セキュリティ・データ整合性・主要機能に関わる乖離 |
| `medium` | エラーハンドリング・境界値・副作用の不一致 |
| `low` | 軽微な表現の違い・補足情報の欠落 |

---

## 6. 制約・留意事項

### 6.1 前提条件

- `spec-weaver trace` が利用可能であること（spec-weaver がインストール済み）
- `claude` CLI が利用可能であること（Claude Code がインストール済み）
- 仕様アイテムに `impl_files` またはコードに `# implements: <ID>` が設定されていること
  （設定がない場合、レビュー対象コードが限定される）

### 6.2 既知の限界

| 限界 | 理由 |
|---|---|
| 実装ファイルが未リンクの場合、見逃しが発生しうる | ファイル特定を精度優先にしているため |
| LLMの判断に誤検知（false positive）が含まれる可能性がある | LLM推論の本質的な限界 |
| 動的に生成されるコード（メタプログラミング等）は評価が困難 | 静的ファイル解析ベースのため |

### 6.3 誤検知への対処

- `severity: low` のfinding は参考情報として扱い、CI の失敗条件には含めない
- `--fail-on high` で CI に組み込む場合は、false positive の可能性を考慮したレビューフローを設ける
- finding は「問題の確定」ではなく「要確認フラグ」として運用する

### 6.4 `audit` との組み合わせ推奨フロー

```bash
# Step 1: 構造・リンクの健全性を確認（必須前提）
uv run spec-weaver audit ./specification/features

# Step 2: 意味的整合性をレビュー（本機能）
uv run spec-weaver review --all --fail-on high
```

---

## 7. 未解決事項（TODO）

| # | 事項 | 優先度 |
|---|---|---|
| T1 | `spec-weaver list-items` コマンドの存在確認または実装 | high |
| T2 | claude CLI の `--file` フラグの挙動・制限の確認 | high |
| T3 | `--all` 時の進捗表示（tqdm等）の要否 | medium |
| T4 | finding の重複排除ロジック（複数ファイル間で同一問題を重複報告しない） | medium |
| T5 | タイムアウト設定（1アイテムのレビューが長時間かかる場合の処理） | medium |
| T6 | レビュー結果のキャッシュ（ファイルが変更されていない場合はスキップ） | low |
