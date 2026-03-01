# 再編ガイド — Doorstop ドキュメントの構造改善手順

Doorstop はアイテムの「移動」コマンドを持たない。
再編とは「新ドキュメントに新アイテムを作り、旧アイテムを非アクティブ化する」プロセスである。

---

## 再編のパターン

### パターン1: フラットドキュメントをサブドキュメントに分割

**Before:**
```
REQ/          # REQ-001〜REQ-020 がフラットに20件
SPEC/         # SPEC-001〜SPEC-018 がフラットに18件
```

**After:**
```
REQ/          # 横断的要件のみ（3件程度）
├── auth/     # AUTH-REQ: 認証ドメイン（6件）
└── pay/      # PAY-REQ: 決済ドメイン（7件）

SPEC/         # 横断的仕様のみ
├── auth/     # AUTH: 認証仕様
└── pay/      # PAY: 決済仕様
```

### パターン2: 大きなドキュメントを `level` でセクション化

ドメイン分割までは不要だが、同一ドキュメント内で整理したい場合。

**Before:** level が全て 1.0 のフラット
**After:** normative: false の見出しアイテムでセクション化

---

## パターン1の実施手順

### フェーズ1: 準備

#### 1-1. 移行対象アイテムのマッピング

マイグレーション表を作成する（コメントや別メモに記録）:

```
旧ID     → 新ドキュメント  新ID（連番）
REQ-002  → AUTH-REQ      AUTH-REQ-001
REQ-005  → AUTH-REQ      AUTH-REQ-002
REQ-008  → AUTH-REQ      AUTH-REQ-003
REQ-003  → PAY-REQ       PAY-REQ-001
REQ-009  → PAY-REQ       PAY-REQ-002
```

#### 1-2. 影響範囲の確認

```bash
# 移行する REQ を参照している SPEC を確認
doorstop
spec-weaver trace REQ-002 -f ./specification/features

# 移行する REQ を参照している feature タグを確認
grep -rn "@REQ-002\|@REQ-005\|@REQ-008" ./specification/features/
```

### フェーズ2: 新ドキュメントの作成

```bash
# 新しいドメインドキュメントを作成
doorstop create AUTH-REQ ./specification/reqs/auth --parent REQ
doorstop create PAY-REQ  ./specification/reqs/payment --parent REQ

# .doorstop.yml の sep を確認して '-' に設定
# （自動生成後に開いて確認）
cat ./specification/reqs/auth/.doorstop.yml
# → settings.sep が '-' であることを確認
```

### フェーズ3: アイテムのマイグレーション

各旧アイテムを1件ずつ移行する。以下を繰り返す:

```bash
# Step A: 新ドキュメントにアイテムを追加
doorstop add AUTH-REQ
# → AUTH-REQ-001.yml が生成される

# Step B: 旧アイテムの内容を新アイテムにコピー
# Read ツールで REQ-002.yml を読み、AUTH-REQ-001.yml の text に転記する
# （level, header, status 等のカスタム属性もコピーする）

# Step C: 旧アイテムを非アクティブ化
# REQ-002.yml を開いて active: false に変更
# active フィールド以外は変更しない（履歴として残す）
```

> **注意**: アイテムを削除（ファイルごと削除）すると Git 履歴が失われる。
> `active: false` で非アクティブ化して残すことを推奨。

### フェーズ4: SPEC のリンク張り直し

```bash
# 旧 REQ-002 を参照していた SPEC のリンクを新 AUTH-REQ-001 に変更

# 旧リンクを削除してから新リンクを張る
# （doorstop link は追加のみ。削除は YAML 直接編集）

# SPEC-005.yml の links フィールドを確認
# links:
# - REQ-002     ← この行を削除して
# - AUTH-REQ-001  ← この行を追加

# 新しいリンクを張る
doorstop link SPEC-005 AUTH-REQ-001

# バリデーションで確認
doorstop
```

### フェーズ5: feature タグの更新

```bash
# 旧タグを検索
grep -rn "@REQ-002" ./specification/features/

# 該当 feature ファイルを編集
# @REQ-002 → @AUTH-REQ-001 に置換
# （feature ファイルは直接テキスト編集可能）
```

### フェーズ6: 検証

```bash
# 全体バリデーション
doorstop

# Gherkin 整合性
spec-weaver audit ./specification/features

# ステータス確認
spec-weaver status
```

エラーがなければ移行完了。

---

## パターン2の実施手順（level セクション化）

フラットなドキュメントに、`normative: false` のセクション見出しアイテムを挿入する。

### Step 1: セクション構成を決める

例: REQ の 20件を以下のセクションに整理
- セクション1 (level 1.x): 認証 → REQ-001, REQ-004, REQ-007
- セクション2 (level 2.x): 決済 → REQ-002, REQ-005, REQ-009
- セクション3 (level 3.x): 通知 → REQ-003, REQ-006

### Step 2: セクション見出しアイテムを追加

```bash
# セクション見出し用のアイテムを追加
doorstop add REQ
```

```yaml
# 生成されたアイテムを開いて編集:
active: true
level: 1.0        # ← セクション番号.0
normative: false  # ← false でヘッダー扱い
header: '認証機能'
text: ''          # ← text は空でよい
links: []
```

### Step 3: 既存アイテムの level を更新

各アイテムの YAML を開いて `level` を書き換える:

```bash
# REQ-001 を認証グループ（level 1.1）に
doorstop edit REQ-001
# → level: 1.1 に変更

# REQ-004 を認証グループ（level 1.2）に
doorstop edit REQ-004
# → level: 1.2 に変更
```

> **注意**: `level` の変更は YAML を直接編集するだけでよい。
> `doorstop add` は新しい連番ファイルを作るだけなので、既存アイテムの level 変更は手動編集。

### Step 4: バリデーション

```bash
doorstop
```

---

## パターン3の実施手順（集約）

### 3-A: ドキュメント集約（複数ドキュメントを1つに統合）

別々のドキュメントに分散しているが、実態は同一ドメインである場合に統合する。

**典型的なケース:**
- `AUTH-REQ`（3件）と `OAUTH-REQ`（2件）が別々だが内容が密接に関連
- マイクロサービス分割を想定して細かく作ったが、一体管理の方が見通しが良い
- 担当チームが変わり、ドメイン境界を再整理する必要がある

#### フェーズ1: 準備

```bash
# 統合元ドキュメントの全アイテムを確認
find ./specification/reqs/oauth -name "*.yml" ! -name ".doorstop.yml"

# 統合元アイテムを参照している SPEC を確認
for id in OAUTH-REQ-001 OAUTH-REQ-002; do
  echo "=== $id を参照する SPEC ==="
  grep -rn "$id" ./specification/specs/
done

# 統合元アイテムを参照している feature タグを確認
grep -rn "@OAUTH-REQ" ./specification/features/
```

#### フェーズ2: アイテムの移行

```bash
# 統合先ドキュメントに新アイテムを追加
doorstop add AUTH-REQ   # → AUTH-REQ-004.yml 生成
# OAUTH-REQ-001.yml の内容（text, status, header 等）をコピー

doorstop add AUTH-REQ   # → AUTH-REQ-005.yml 生成
# OAUTH-REQ-002.yml の内容をコピー
```

#### フェーズ3: 統合元アイテムの非アクティブ化

```yaml
# OAUTH-REQ-001.yml（統合元）
active: false
migrated_to: AUTH-REQ-004   # ← カスタム属性でトレーサビリティを保持
status: deprecated
```

#### フェーズ4: リンクと feature タグの更新

```bash
# SPEC のリンクを張り直す
# SPEC-xxx.yml の links: [OAUTH-REQ-001] → [AUTH-REQ-004] に変更
doorstop link SPEC-xxx AUTH-REQ-004
# YAML から OAUTH-REQ-001 を削除

# feature タグを更新
grep -rn "@OAUTH-REQ" ./specification/features/
# @OAUTH-REQ-001 → @AUTH-REQ-004, @OAUTH-REQ-002 → @AUTH-REQ-005
```

#### フェーズ5: 空になったドキュメントの処理

アイテムが全て非アクティブ化されたドキュメントは2つの選択肢がある:

**選択肢A: そのまま残す（推奨）**
`.doorstop.yml` は残し、active アイテムが 0 件の状態にする。
Doorstop の警告は出るが、Git 履歴が保持される。

**選択肢B: ドキュメントごと削除**
```bash
# 非アクティブアイテムを全て確認してから削除
rm -rf ./specification/reqs/oauth/
# Git 履歴には削除が記録される
doorstop  # バリデーション（リンク切れがないか確認）
```

#### フェーズ6: 検証

```bash
doorstop && spec-weaver audit ./specification/features
```

---

### 3-B: アイテム集約（複数アイテムを1つに統合）

粒度が細かすぎる複数のアイテムを、1つの包括的なアイテムにまとめる。

**典型的なケース:**
- `SPEC-003`「ログインボタンを押す」と `SPEC-004`「ログイン処理が実行される」が1つのシナリオで表現できる
- 実装してみたら複数 SPEC が同一コンポーネントを指していると判明した
- 細かい SPEC が増えすぎて feature ファイルのタグが煩雑になった

#### 手順

```bash
# Step 1: 統合先アイテムを特定（または新規作成）
# 既存の SPEC-003 を残して SPEC-007 を統合する場合:

# Step 2: 統合先の text を更新（両方の内容を包含する形で記述）
# SPEC-003.yml の text を編集して SPEC-007 の内容も含める

# Step 3: 統合元を非アクティブ化
# SPEC-007.yml:
#   active: false
#   migrated_to: SPEC-003
#   status: deprecated

# Step 4: 統合元 SPEC-007 を参照するリンクを更新
grep -rn "SPEC-007" ./specification/
# 各ファイルの links から SPEC-007 を削除し、必要なら SPEC-003 を追加

# Step 5: feature タグを更新
grep -rn "@SPEC-007" ./specification/features/
# @SPEC-007 を削除（SPEC-003 のシナリオに統合済みのため）

# Step 6: バリデーション
doorstop && spec-weaver audit ./specification/features
```

---

## パターン4の実施手順（階層変更）

Doorstop の親子関係は各ドキュメントの `.doorstop.yml` に記録されている。
直接編集することで変更できる。

```yaml
# .doorstop.yml の構造
settings:
  digits: 3
  parent: REQ      # ← この parent を変更する
  prefix: NOTIFY-REQ
  sep: '-'
```

### 4-A: 既存ドキュメントの親を変更

**Before:**
```
REQ
├── AUTH-REQ
└── NOTIFY-REQ   ← REQ 直属だが、MESSAGING の下が適切
```

**After:**
```
REQ
├── AUTH-REQ
└── MESSAGING-REQ（新規）
    └── NOTIFY-REQ
```

#### 手順

```bash
# Step 1: 中間ドキュメントが必要なら作成
doorstop create MESSAGING-REQ ./specification/reqs/messaging --parent REQ
# .doorstop.yml の sep: '-' を確認

# Step 2: 移動するドキュメントの .doorstop.yml を編集
# ./specification/reqs/notify/.doorstop.yml:
# settings:
#   parent: REQ          ← 変更前
#   parent: MESSAGING-REQ  ← 変更後

# Step 3: バリデーション
doorstop

# Step 4: 必要なら NOTIFY-REQ のアイテムのリンクも更新
# （NOTIFY-REQ アイテムが REQ アイテムにリンクしている場合、
#  MESSAGING-REQ アイテムにリンクし直すか検討する）
```

### 4-B: サブドキュメントをルートに昇格

```bash
# ./specification/reqs/auth/.doorstop.yml を編集:
# settings:
#   parent: REQ     ← 削除またはブランクにする
#   parent: ''      ← ルートドキュメントになる

doorstop  # バリデーション
```

> **注意**: ルートドキュメントが複数になる場合、Doorstop はそれぞれ独立したツリーとして扱う。
> ルートは通常 1つにまとめることが推奨される。

### 4-C: フラットな兄弟ドキュメントを階層化

多くのドキュメントが同じ親を持つ「フラットな兄弟」構成を、
中間層を挿入して階層化する。

**Before:**
```
REQ
├── AUTH-REQ
├── OAUTH-REQ
├── SESSION-REQ
├── PAY-REQ
├── INVOICE-REQ
└── NOTIFY-REQ
```

**After:**
```
REQ
├── IDENTITY-REQ（新規：認証系をまとめる）
│   ├── AUTH-REQ
│   ├── OAUTH-REQ
│   └── SESSION-REQ
├── PAYMENT-REQ（新規：決済系をまとめる）
│   ├── PAY-REQ
│   └── INVOICE-REQ
└── NOTIFY-REQ
```

#### 手順

```bash
# Step 1: 中間ドキュメントを作成
doorstop create IDENTITY-REQ ./specification/reqs/identity --parent REQ
doorstop create PAYMENT-REQ  ./specification/reqs/payment-group --parent REQ

# Step 2: 各子ドキュメントの parent を変更
# ./specification/reqs/auth/.doorstop.yml: parent: IDENTITY-REQ
# ./specification/reqs/oauth/.doorstop.yml: parent: IDENTITY-REQ
# ./specification/reqs/session/.doorstop.yml: parent: IDENTITY-REQ
# ./specification/reqs/pay/.doorstop.yml: parent: PAYMENT-REQ
# ./specification/reqs/invoice/.doorstop.yml: parent: PAYMENT-REQ

# Step 3: バリデーション
doorstop

# Step 4: 中間ドキュメントにアイテムを追加するか検討
# IDENTITY-REQ にルートとなる横断的な要件アイテムを作成してもよい
# （なくても構造上の問題はない）
```

---

## よくある問題と対処法

### 問題: `doorstop link` で "already linked" エラー

既にリンクが張られている場合。重複リンクは問題ないが、
旧リンクを削除したい場合は YAML の `links` フィールドを直接編集する。

### 問題: 新 ID と旧 ID が混在して混乱する

移行中は旧アイテムに以下のカスタム属性を追記しておくと追跡しやすい:

```yaml
# 旧 REQ-002.yml
active: false
migrated_to: AUTH-REQ-001  # カスタム属性として記録
status: deprecated
```

### 問題: feature タグが多数あって一括置換したい

```bash
# sed で一括置換（バックアップを取ってから実行）
cp -r ./specification/features/ ./specification/features.bak/
sed -i 's/@REQ-002/@AUTH-REQ-001/g' ./specification/features/*.feature
```

### 問題: doorstop バリデーションが通らない

```bash
# 詳細なエラーを確認
doorstop --verbose

# よくある原因:
# 1. 旧アイテムへのリンクが残っている → YAML の links を更新
# 2. .doorstop.yml の sep が '-' 以外 → sep: '-' を設定
# 3. 非アクティブアイテムが親リンクのみの SPEC に参照されている
```

---

## 再編後のコミット規約

```bash
# 再編内容を明確に記述
git add ./specification/
git commit -m "refactor(spec): REQをドメイン別サブドキュメントに再編

- AUTH-REQ: 認証ドメイン（REQ-002,005,008 を移行）
- PAY-REQ: 決済ドメイン（REQ-003,009 を移行）
- 旧アイテムは active: false で保持（履歴のため）"
```
