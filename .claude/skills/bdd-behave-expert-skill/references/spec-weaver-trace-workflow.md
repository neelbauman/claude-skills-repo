# spec-weaver trace を使ったBDDテスト設計ワークフロー

## なぜ trace から始めるのか

BDDテストを書くには「何を証明するか」が明確でなければならない。
spec-weaver trace はその問いに答えるための情報を、単一コマンドで収集できる。

- **REQ** → なぜこの振る舞いが必要か（背景・目的）
- **SPEC** → 何を実装すべきか（仕様本文）
- **`.feature`** → 既にどんな振る舞いがテストされているか
- **実装ファイル** → 現在どう実装されているか

この4つを把握してから Feature / Step を設計する。

---

## Step 1: 起点の特定

### SPEC ID がわかっている場合

```bash
uv run spec-weaver trace SPEC-010 -f ./specification/features --show-impl
```

### Feature ファイルを起点にする場合

```bash
# feature → SPEC → REQ の順に遡る
uv run spec-weaver trace checkout.feature -f ./specification/features --direction up
```

### 全体像を把握したい場合

```bash
# REQ-001 を起点に全子孫を展開する
uv run spec-weaver trace REQ-001 -f ./specification/features --direction down
```

---

## Step 2: trace 出力の読み方

### 典型的な出力例

```
REQ-005 決済フローの整合性保証 ✅ implemented
└── ★ SPEC-010 送料計算ルール ✅ implemented
    ├── 🥒 checkout.feature
    │   ├── Scenario Outline: VIPユーザーの購入金額別送料 -- @1.1 通常送料
    │   └── Scenario Outline: VIPユーザーの購入金額別送料 -- @1.2 送料無料ライン到達
    ├── 📁 src/checkout/shipping.py
    └── 📝 src/checkout/cart.py
```

### 読み解き方

| 記号 | 確認すべきこと |
|---|---|
| `★ SPEC-010` | この SPEC の `text` フィールドを読んで仕様本文を把握する |
| 親 REQ | なぜこのルールが必要かの文脈を確認する |
| `🥒` + シナリオ | 既存のシナリオで何がカバーされているか確認する |
| `📁` `📝` 実装ファイル | 現在の実装を読んで仕様との乖離を確認する |

---

## Step 3: Doorstop YAML の仕様本文を読む

trace で SPEC ID が判明したら、YAML を直接読んで仕様本文を確認する。

```bash
# YAML の場所を確認する
find ./specification -name "SPEC-010.yml"

# または直接読む
cat ./specification/specs/SPEC-010.yml
```

```yaml
# SPEC-010.yml の例
active: true
status: implemented
testable: true
links:
  - REQ-005: abc123
text: |
  ## 送料計算ルール

  VIPユーザーが商品を購入する際の送料は以下のルールに従う。

  - 購入合計金額が 10,000 円未満の場合: 送料 500 円
  - 購入合計金額が 10,000 円以上の場合: 送料無料（0 円）

  送料計算は商品をカートに追加した時点でリアルタイムに更新されること。
```

**この `text` が「あるべき振る舞い」の唯一の根拠**。
実装ファイルではなく、この YAML を基準にテストを設計する。

---

## Step 4: 既存テストとのギャップ分析

trace で確認できた既存シナリオと仕様本文を照合する。

### チェックリスト

- [ ] 仕様に記載されたすべての条件分岐がシナリオでカバーされているか
- [ ] 境界値（10,000 円ちょうど、9,999 円、10,001 円）が含まれているか
- [ ] 異常系（不正な入力、存在しないユーザーなど）が記述されているか
- [ ] `testable: true` の仕様に `.feature` タグが付いているか

### audit でギャップを定量確認

```bash
uv run spec-weaver audit ./specification/features
```

出力例:
```
❌ テストが実装されていない仕様 (Untested Specs):
  SPEC-010
```

---

## Step 5: 仕様と実装の乖離の扱い方

### 乖離パターンと対応

| 状況 | 対応 |
|---|---|
| 実装が仕様通りに動いていない | テストで乖離を固定化する（テストを失敗させる） |
| 実装が仕様を超えた振る舞いをしている | 仕様にない振る舞いもテストに加えるが SPEC の更新を提案する |
| 仕様が曖昧で実装と合わせるしかない | 仕様の明確化をユーザーに確認する。憶測でテストを書かない |
| `testable: false` の仕様 | Gherkin でのテストは不要（UI見た目、設定値など） |

### テストで乖離を固定化する例

```gherkin
# SPEC-010 の仕様: 10,000 円以上で送料無料
# 現在の実装: 15,000 円以上でないと送料無料にならない（バグ）

@SPEC-010
Scenario: 10000円ちょうどで送料無料になること（仕様準拠）
  Given VIPユーザーとしてログインしている
  When カートに 2 個の "商品A" (単価: 5000円) を追加する
  Then 送料は 0 円になること   # ← 現在の実装ではここで失敗する（正しい）
```

このテストは **意図的に失敗させる**。
失敗が「仕様に追従していない実装」を可視化する。

---

## Step 6: audit で最終確認

テストを書いたら必ず audit でリンク整合性を確認する。

```bash
# Gherkin タグと Doorstop のリンク整合性
uv run spec-weaver audit ./specification/features

# 実装ファイルのリンクも含めて確認する場合
uv run spec-weaver audit ./specification/features --check-impl
```

---

## trace コマンド クイックリファレンス

```bash
# 基本（both: 上位+下位を展開）
uv run spec-weaver trace SPEC-xxx -f ./specification/features

# 上位のみ（REQ まで遡る）
uv run spec-weaver trace SPEC-xxx -f ./specification/features --direction up

# 下位のみ（シナリオまで展開）
uv run spec-weaver trace REQ-xxx -f ./specification/features --direction down

# 実装ファイルも表示
uv run spec-weaver trace SPEC-xxx -f ./specification/features --show-impl

# テーブル形式で出力
uv run spec-weaver trace SPEC-xxx -f ./specification/features --format flat

# .feature ファイルを起点に遡る
uv run spec-weaver trace target.feature -f ./specification/features --direction up
```
