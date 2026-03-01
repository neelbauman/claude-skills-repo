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
