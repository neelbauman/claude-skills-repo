# behave 実装パターン集

## scaffold による雛形生成（出発点）

Step 定義は必ず `spec-weaver scaffold` で雛形を生成してから始める。

```bash
uv run spec-weaver scaffold ./specification/features --out-dir features/steps
```

### 生成されたファイルの読み方

```python
# scaffold が生成する雛形の例（step_checkout.py）

# 使用されるシナリオ:
# - VIPユーザーの購入金額別送料
@when('カートに "{param0}" (単価: "{param1}") を追加する')  # type: ignore
def when_a1b2c3d4(context, param0, param1):
    """カートに "高級イヤホン" (単価: 5000円) を追加する"""
    raise NotImplementedError('STEP: カートに "{param0}" (単価: "{param1}") を追加する')
```

- **コメント行**: このステップがどのシナリオで使われているかを示す
- **デコレータ**: `"..."` は `{paramN}` にパラメータ化済み（型なし）
- **関数名**: SHA256 ハッシュ（非 ASCII 文字回避のため）
- **docstring**: 元のステップ文（参照用）
- **本体**: `raise NotImplementedError` — ここを仕様に従って実装する

### 雛形から仕様に合わせて書き換える

変更してよいのは以下のみ:

- **❶** デコレータの型なし `{paramN}` → 型付きパラメータ（`{count:d}` 等）
- **❷** 本体の `raise NotImplementedError` → 委譲コード・`assert`

**以下は絶対に削除しない:**

- `# type: ignore`（デコレータ末尾）— Pyright の誤検知を防ぐ
- `# 使用されるシナリオ:` / `# - シナリオ名` — トレーサビリティコメント
- docstring（元のステップ文）— Gherkin 原文の記録

```python
# Before（scaffold 生成）
# 使用されるシナリオ:
# - VIPユーザーの購入金額別送料
@when('カートに "{param0}" (単価: "{param1}") を追加する')  # type: ignore
def when_a1b2c3d4(context, param0, param1):
    """カートに "高級イヤホン" (単価: 5000円) を追加する"""
    raise NotImplementedError('STEP: カートに "{param0}" (単価: "{param1}") を追加する')

# After（仕様に従って実装 — コメント・type: ignore・docstring は残す）
# 使用されるシナリオ:
# - VIPユーザーの購入金額別送料
@when('カートに {count:d} 個の "{item}" (単価: {price:d}円) を追加する')  # type: ignore
def when_a1b2c3d4(context, count, item, price):
    """カートに "高級イヤホン" (単価: 5000円) を追加する"""
    context.api_client.add_items_to_cart(name=item, unit_price=price, quantity=count)
```

---

## Step 定義の基本構造

### 正しい Step（Thin Wrapper の例）

```python
from behave import given, when, then


# ✅ 良い: 受け取って渡すだけ
@given('ユーザー "{username}" としてログインしている')
def step_impl(context, username):
    context.api_client.login(username)


# ✅ 良い: 型付きパラメータで受け取る
@when('カートに {count:d} 個の "{item}" を追加する')
def step_impl(context, count, item):
    context.api_client.add_to_cart(item, quantity=count)


# ✅ 良い: assert だけ
@then('注文総額は {expected:d} 円であること')
def step_impl(context, expected):
    actual = context.api_client.get_order_total()
    assert actual == expected
```

### 禁止パターン（Anti-Pattern）

```python
# ❌ 悪い: Step 内で計算している
@then('送料無料になること')
def step_impl(context):
    total = context.cart_items * context.unit_price   # ← 計算はここでしない
    shipping = 0 if total >= 10000 else 500            # ← ビジネスロジックの再実装
    assert context.api_client.get_shipping() == shipping


# ❌ 悪い: 条件分岐がある
@when('カートを確認する')
def step_impl(context):
    if context.user_status == 'VIP':   # ← if は禁止
        context.api_client.apply_vip_discount()
    context.api_client.view_cart()


# ❌ 悪い: 正規表現パースを使っている
import re

@when('"{item}" を {count} 個追加して合計 {total} 円になる場合')
def step_impl(context, item, count, total):
    count = int(re.sub(r'[^\d]', '', count))   # ← 自前パースは禁止
```

---

## パラメータ型の正規ルート

### 型付きパラメータ（Typed Parameters）

```python
# 整数
@then('残高は {amount:d} 円であること')
def step_impl(context, amount):
    assert context.account.balance == amount

# 浮動小数点
@then('割引率は {rate:f} ％であること')
def step_impl(context, rate):
    assert context.discount.rate == rate

# 文字列（デフォルト）
@given('ステータスが "{status}" のユーザーとしてログインしている')
def step_impl(context, status):
    context.api_client.login_as(status=status)
```

### Data Table の受け取り

```python
# context.table でそのままアクセスする
@given('システムに以下のユーザーが登録されている:')
def step_impl(context):
    users = [
        {"username": row["username"], "status": row["status"]}
        for row in context.table
    ]
    context.api_client.setup_users(users)


# ヘッダーなしの場合（rows）
@when('以下のアイテムをカートに追加する:')
def step_impl(context):
    items = [row[0] for row in context.table.rows]
    context.api_client.add_items(items)
```

### Doc String の受け取り

```python
# context.text で受け取る
@then('エラーメッセージは以下であること:')
def step_impl(context):
    expected = context.text.strip()
    actual = context.api_client.get_last_error()
    assert actual == expected
```

---

## environment.py のパターン

### 標準構成

```python
# features/environment.py

def before_all(context):
    """テスト全体の初期化"""
    context.base_url = "http://localhost:8000"
    # context.xxx に API クライアントを設定
    context.api_client = MyApiClient(base_url=context.base_url)


def before_scenario(context, scenario):
    """シナリオごとのリセット"""
    context.api_client.reset()


def after_scenario(context, scenario):
    """シナリオ後のクリーンアップ"""
    context.api_client.cleanup()
```

### ルール: environment.py に書くもの

| 書いてよいもの | 書いてはいけないもの |
|---|---|
| クライアントの初期化 | ビジネスロジック |
| DB 接続 / リセット | アサーション |
| 認証トークンの設定 | テストデータの準備（Step に書く） |

---

## Scenario Outline のパターン

### 正しい Examples 分割

```gherkin
# ✅ 良い: 境界値ごとに Examples を分割し、名前で意味を表現する
Scenario Outline: ユーザーランク別割引率
  Given ユーザーのランクは "<ランク>" である
  Then 割引率は <割引率> ％であること

  Examples: 一般会員
    | ランク  | 割引率 |
    | NORMAL  | 0      |

  Examples: 優良会員
    | ランク  | 割引率 |
    | SILVER  | 5      |
    | GOLD    | 10     |

  Examples: VIP会員（特別扱い）
    | ランク  | 割引率 |
    | VIP     | 20     |
```

```gherkin
# ❌ 悪い: 意味の異なるケースを1つの Examples に混在させる
Examples: パターン
  | ランク  | 割引率 |
  | NORMAL  | 0      |
  | VIP     | 20     |   # ← 区別ができない
```

---

## Step 定義ファイルの分割ルール

### ファイル構成

```
features/
├── steps/
│   ├── common_steps.py      # 複数 Feature で共通する Given（ユーザー登録など）
│   ├── checkout_steps.py    # checkout.feature 専用
│   ├── auth_steps.py        # auth.feature 専用
│   └── inventory_steps.py   # inventory.feature 専用
└── environment.py
```

### 共有 Step の条件

- 複数の Feature でまったく同じ文言の Step が必要な場合のみ共有する
- 文言が似ていても意味が違う場合は別の Step として定義する
- Step を共有する場合は `common_steps.py` に集約し、ファイル名で明示する

---

## テスト失敗の扱い

### 失敗は「設計の意思表示」

- 実装が未完成でテストが落ちる → **正常。実装を直すべき**
- 仕様が変わってテストが落ちる → **正常。仕様の変更を反映すべき**
- テストが間違っていてテストが落ちる → **テストを修正する（仕様に合わせる）**

### @wip タグによる段階的実装

```gherkin
@wip
Scenario: まだ実装されていないシナリオ
  Given 未実装の前提条件
  When 未実装のアクション
  Then 未実装の期待結果
```

```bash
# @wip タグ付きだけ実行する
behave --tags=wip

# @wip タグを除外して実行する
behave --tags=~wip
```

`@wip` は「実装予定」を示す一時的なタグ。実装が完了したら必ず削除すること。

---

## behave の実行コマンド

```bash
# 全シナリオ実行
uv run behave specification/features/

# 特定の .feature だけ実行
uv run behave specification/features/checkout.feature

# タグ指定で実行
uv run behave --tags=SPEC-010 specification/features/

# 詳細出力
uv run behave --no-capture specification/features/

# Cucumber JSON 出力（spec-weaver build で使用）
uv run behave --format json -o test-results.json specification/features/
```
