---
name: doorstop-gherkin-spec
description: >
  Doorstop（テキストベースの要件管理CLI）とGherkin（.feature形式の振る舞い仕様）、
  およびそれらを繋ぐSpec-WeaverというCLIツールを組み合わせた、仕様管理プロセスのサポートスキル。
  新規プロジェクトのDoorstop初期化・YAML作成・featureファイル生成から、要件・仕様・featuresの更新に伴う整合性更新、
  既存プロジェクトのコードベース分析による仕様の逆引き初期化まで対応する。
  ユーザーが「仕様管理」「仕様更新」「Doorstop」「Gherkin」「要件定義」「.feature」「BDD」
  「受け入れ条件」「Spec-Weaver」「トレーサビリティ」を話題にした場合、
  または既存プロジェクトに仕様管理を導入・整備したい場合は必ずこのスキルを使うこと。
---

# Doorstop + Gherkin + Spec-Weaver 仕様管理スキル

## ツール・役割の分担

| レイヤー | ツール | 問い | 形式 |
|---|---|---|---|
| ビジネス要件 | Doorstop (`specification/reqs/`) | なぜ作るのか | YAML |
| システム仕様 | Doorstop (`specification/specs/`) | 何を作るのか | YAML |
| 振る舞い仕様 | Gherkin (`specification/features/`) | どう振る舞うか | .feature |
| 整合性チェック | **Spec-Weaver** | 仕様とテストに乖離がないか | CLI |
| テスト実装 | 各言語フレームワーク | 実際に動くか | コード（スコープ外） |

**⚠️ 重要: DoorstopのYAMLは手動で作ってはいけない。必ず `doorstop` CLIで生成する。**

---

## Doorstop CLIの正しい使い方

### インストール

```bash
uv tool install doorstop
```

### ドキュメントの作成（`doorstop create`）

```bash
# ルートドキュメント（親なし）を作成
doorstop create REQ ./specification/reqs

# 子ドキュメントを作成（--parent で親プレフィックスを指定）
doorstop create SPEC ./specification/specs --parent REQ
```

これにより各ディレクトリに `.doorstop.yml` が自動生成される。Gitリポジトリのルートで実行すること。

### アイテムの追加（`doorstop add`）

```bash
# REQアイテムを追加（REQ-001.yml が自動生成される）
doorstop add REQ

# 生成された YAML ファイルをエディタで開いて text を編集する
doorstop edit REQ-001
```

**生成されるYAMLの初期構造:**
```yaml
active: true
derived: false
header: ''
level: 1.0
links: []
normative: true
ref: ''
reviewed: null
text: |
  （ここに要件/仕様の本文を記述する）
```

### アイテムのリンク（`doorstop link`）

```bash
# SPEC-001 を REQ-001 にリンク
doorstop link SPEC-001 REQ-001
```

これにより `SPEC-001.yml` の `links` フィールドが自動更新される。

### 検証とレビュー

```bash
# ツリー全体の整合性チェック（リンク切れ・未レビュー検出）
doorstop

# アイテムをレビュー済みにする（fingerprintが記録される）
doorstop review all

# HTMLとして公開
doorstop publish all ./specification/public
```

---

## ID形式の設定（Spec-Weaverとの連携）

Spec-Weaverはデフォルトで `SPEC-001` 形式（ダッシュ区切り）を想定している。
Doorstop作成時は以下のように `sep` を指定する:

```bash
# sep はドキュメント作成後に .doorstop.yml を直接編集して設定
# もしくは create 後に .doorstop.yml を確認・修正する
```

`.doorstop.yml` の中身（自動生成後に確認）:
```yaml
settings:
  digits: 3
  prefix: SPEC
  sep: '-'     # ← Spec-Weaverに合わせてダッシュを使う
```

---

## Spec-Weaverの使い方

references/how-to-use-spec-weaver.md を参照。

---

## ドキュメント構造の設計指針

**⚠️ REQ/SPECをフラットに追加し続けると、アイテムが増えるにつれて「関連する仕様のまとまり」が見えにくくなる。**
グループ化戦略を事前に決めておくことが重要。

### サブドキュメント vs `level` フィールド — どちらを使うか

| 手段 | 使うタイミング |
|---|---|
| **サブドキュメント**（別prefix） | ドメイン・機能領域が明確に分離している／1ドキュメントが10件超える |
| **`level` フィールド** | 同一ドメイン内で小さなサブグループを作る／アイテムが10件以下 |

#### `level` + `normative: false` で文書内セクションを作る方法

```yaml
# REQ-001.yml（セクション見出し）
active: true
level: 1.0          # .0 終わりがセクション
normative: false    # false にするとヘッダー扱い
header: '認証機能'
text: ''

# REQ-002.yml（実際の要件：認証グループ）
active: true
level: 1.1
normative: true
text: |
  ユーザーはメールアドレスとパスワードでログインできること。

# REQ-003.yml（別ドメインのセクション見出し）
active: true
level: 2.0
normative: false
header: '決済機能'
text: ''
```

> `level` の `.0` 終わりアイテムを `normative: false` にすると、Doorstop はそれをセクション見出しとして扱う。
> 同一ドキュメント内で複数のドメインを管理する際に有効。

#### サブドキュメント構成（ドメインが明確に分かれる場合）

```bash
doorstop create REQ      ./specification/reqs              # ルート要件（横断的）
doorstop create AUTH-REQ ./specification/reqs/auth --parent REQ    # 認証ドメイン
doorstop create PAY-REQ  ./specification/reqs/payment --parent REQ # 決済ドメイン

doorstop create SPEC     ./specification/specs             # ルート仕様（横断的）
doorstop create AUTH     ./specification/specs/auth --parent AUTH-REQ
doorstop create PAY      ./specification/specs/payment --parent PAY-REQ
```

#### 判断フロー

```
アイテムが増えてきた
    ↓
機能領域が明確に分かれる? → Yes → サブドキュメント（別prefix）を作成
    ↓ No
同一ドキュメントが10件超える? → Yes → level + normative:false でセクション化
    ↓ No
そのままフラットで管理
```

> **制約**: Doorstopの `--parent` は1つだけ指定可能（多重継承不可）。
> 複数ドメインにまたがる仕様は、上位REQへのリンクを複数張ることで対応する（`doorstop link SPEC-001 REQ-002`）。

---

## ディレクトリ構成（標準テンプレート）

```text
<project-root>/specification/
├── reqs/                  # ビジネス要件 [Doorstop: prefix=REQ]
│   ├── .doorstop.yml      # doorstop create REQ ./specification/reqs で自動生成
│   ├── REQ-001.yml
│   └── auth/              # サブグループ（認証ドメイン）
│       ├── .doorstop.yml  # doorstop create AUTH-REQ ./specification/reqs/auth --parent REQ
│       └── AUTH-REQ-001.yml
├── specs/                 # システム仕様 [Doorstop: prefix=SPEC, parent=REQ]
│   ├── .doorstop.yml      # doorstop create SPEC ./specification/specs --parent REQ で自動生成
│   ├── SPEC-001.yml
│   └── auth/              # サブグループ（認証ドメイン）
│       ├── .doorstop.yml  # doorstop create AUTH ./specification/specs/auth --parent AUTH-REQ
│       └── AUTH-001.yml
└── features/              # 振る舞い仕様 [Gherkin]
    ├── auth.feature       # @AUTH-001 タグで紐付け
    └── payment.feature    # @PAY-001 タグで紐付け
```

---

## モード1: 新規プロジェクトのセットアップ

### Step 0: ドメイン設計（必須・Doorstop初期化より前に行う）

Doorstopを初期化する前に、仕様の「ドメイン（機能領域）」を設計する。
**この手順を省略すると、アイテムが無秩序に増殖し、後から再整理が困難になる。**

#### ドメインの識別

ユーザーと以下を確認する:
- **主要機能領域**: 認証、決済、通知、ユーザー管理、在庫管理 など
- **横断的要件**: 全ドメインに共通するREQ（パフォーマンス、セキュリティ方針 など）
- **規模感の見積もり**: 各ドメインのアイテム数が5件以下か、それ以上か

#### ドキュメント構造の決定

| 状況 | 推奨構成 |
|---|---|
| 全体で5〜10件程度の小規模プロジェクト | REQ + SPEC のフラット構成 + `level` セクション化 |
| 機能領域が2〜3つ、各10件以内 | ドメイン別サブドキュメント（AUTH-REQ/AUTH など） |
| 機能領域が4つ以上 または 各領域が10件超える見込み | 必ずサブドキュメントで階層化 |

#### ドメイン設計の出力（ユーザーに確認してから次へ）

```text
ドメイン設計:
  横断: REQ（ルート）/ SPEC（ルート）
  認証: AUTH-REQ / AUTH
  決済: PAY-REQ  / PAY
  通知: NTF-REQ  / NTF

グループ化戦略:
  小規模ドメインは level フィールドでセクション化
  大規模ドメインはサブドキュメント化
```

### Step 1: Doorstop初期化

```bash
cd <project-root>
doorstop create REQ ./specification/reqs
doorstop create SPEC ./specification/specs --parent REQ

# .doorstop.yml の sep を '-' に修正（Spec-Weaver対応）
# specification/reqs/.doorstop.yml と specs/.doorstop.yml を確認して sep: '-' を追加
```

### Step 2: 要件を追加・編集

```bash
doorstop add REQ   # → REQ-001.yml 生成
doorstop edit REQ-001  # エディタで text を記述
```

### Step 3: 仕様を追加・REQにリンク

```bash
doorstop add SPEC         # → SPEC-001.yml 生成
doorstop link SPEC-001 REQ-001  # リンク設定
doorstop edit SPEC-001    # エディタで text を記述
```

### Step 4: Gherkin .featureの作成

詳細は `references/gherkin-guide.md` を参照。

### Step 5: 整合性チェック

```bash
doorstop            # Doorstop内のリンク整合性
spec-weaver audit ./specification/features  # Doorstop ↔ Gherkin の整合性
```

---

## モード2: 既存プロジェクトの分析と逆引き初期化

### 分析フロー

**1. コードベースのスキャン**
```bash
find . -type f \( -name "*.py" -o -name "*.ts" -o -name "*.tsx" \) | head -60
find . -name "README*" -o -name "*.md" | head -20
```

**2. 機能領域の推定**
- ルーティング定義（routes/, pages/, controllers/）からエンドポイントを抽出
- 既存テストの `describe`/`it` ブロックのテキストはGherkin化しやすい

**3. 仕様ドラフトの生成手順**
```bash
# まずDoorstopを初期化
doorstop create REQ ./specification/reqs
doorstop create SPEC ./specification/specs --parent REQ

# 機能ごとにアイテムを追加
doorstop add REQ  # 機能の数だけ繰り返す
```

その後、生成されたYAMLファイルの `text` フィールドに分析した内容を記述し、
`doorstop link` でREQ-SPECの紐付けを行う。

### 逆引き時の注意

- 実装から推測できるのは「何をしているか（SPEC）」まで。「なぜ（REQ）」はユーザーに必ず確認
- 完璧な仕様より「まず存在する仕様」を優先し、後から精緻化を提案する
- `testable: false` を付けるべき仕様（UI見た目、設定値など）も整理する

---

---

## 実装ステータスの管理

Doorstop YAMLに `status` カスタム属性を追記することで、実装の進行状況を管理できる。

### ステータス値

| 値 | バッジ | 意味 |
|---|---|---|
| `draft` | 📝 draft | 草案。まだ実装着手していない |
| `in-progress` | 🚧 in-progress | 実装中 |
| `implemented` | ✅ implemented | 実装済み |
| `deprecated` | 🗑️ deprecated | 廃止予定 |

### ステータスの書き方

`doorstop edit <ID>` でYAMLを開き、`status` キーを追記する:

```yaml
active: true
status: in-progress   # ← この行を追記
text: |
  （仕様本文）
```

### タスク終了時のステータス更新手順

実装作業が終わったら、関連する REQ/SPEC の `status` を更新すること:

1. **実装が完了した SPEC を確認する**
   ```bash
   spec-weaver status --filter in-progress
   ```

2. **完了した SPEC の YAML を更新する**
   ```bash
   # SPEC-001.yml の status を implemented に変更
   # （doorstop edit SPEC-001 でエディタを開くか、直接 YAML を編集）
   ```
   ```yaml
   status: implemented
   ```

3. **ステータス一覧で確認**
   ```bash
   spec-weaver status
   ```

4. **build でドキュメントに反映**
   ```bash
   spec-weaver build ./specification/features --out-dir .specification
   ```

### ステータス表示コマンド

```bash
# 全アイテムのステータス一覧
spec-weaver status

# 特定ステータスで絞り込み
spec-weaver status --filter draft
spec-weaver status --filter in-progress
spec-weaver status --filter implemented
spec-weaver status --filter deprecated
```

---

## 参照ファイル

| ファイル | 読むタイミング |
|---|---|
| `references/yaml-templates.md` | REQ/SPEC YAMLの内容を編集するとき |
| `references/gherkin-guide.md` | .featureファイルを作成・編集するとき |
| `references/ci-integration.md` | GitHub ActionsやPre-commitの設定をするとき |
| `references/how-to-use-spec-weaver.md` | DoorstopとGherkinの整合性を確認する時 |
