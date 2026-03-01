---
name: doorstop-review
description: >
  Doorstop + Gherkin 仕様管理のドキュメントレビューと再編スキル。
  増えたドキュメントが整合性をもって記述・配置されているかをチェックし、
  必要があれば再整理（分割・集約・階層変更）を提案・実行する。
  ユーザーが「仕様を見直したい」「ドキュメントが増えてきた」「構造を整理したい」
  「仕様のレビュー」「再編」「整合性チェック」「ドキュメント整理」
  「分割」「集約」「統合」「階層変更」「親を変える」を話題にした場合は
  必ずこのスキルを使うこと。
---

# Doorstop 仕様ドキュメント レビュー & 再編スキル

## スキルの目的

Doorstop + Gherkin で管理された仕様ドキュメントが、以下の観点で健全かを診断し、
問題があれば段階的に再整理する。

| 観点 | チェックする問い |
|---|---|
| 構造 | ドキュメント・アイテムが適切に階層化されているか |
| 整合性 | リンク切れ・孤立アイテムがないか |
| 内容 | 重複・矛盾・不完全な記述がないか |
| 鮮度 | 廃止済みの仕様が `active: false` になっているか |
| Gherkin連携 | feature ファイルとのタグ対応が取れているか |

---

## モード1: レビュー（診断のみ）

ドキュメントを変更せず、問題点を報告する。

### Step 1: 構造スキャン

```bash
# Doorstop ドキュメントツリーの全体像を把握
find ./specification -name ".doorstop.yml" | sort

# 各ドキュメントのアイテム数を確認
find ./specification -name "*.yml" ! -name ".doorstop.yml" | \
  awk -F'/' '{print $NF}' | sed 's/-[0-9]*.yml//' | sort | uniq -c | sort -rn
```

### Step 2: Doorstop バリデーション

```bash
# リンク切れ・未レビューアイテムを検出
doorstop

# 全アイテムのステータス一覧
spec-weaver status
```

### Step 3: Gherkin 整合性チェック

```bash
# SPEC と .feature の対応チェック
spec-weaver audit ./specification/features

# 孤立した SPEC（featureで参照されていないもの）を確認
spec-weaver audit ./specification/features --show-untested
```

### Step 4: ファイル内容の分析

以下を Grep / Read ツールで確認する:

```bash
# status フィールドが未設定のアイテム（管理されていない）
grep -rL "^status:" ./specification/reqs ./specification/specs 2>/dev/null

# active: false なアイテム（廃止済み・整理候補）
grep -rl "^active: false" ./specification/

# links が空のアイテム（孤立アイテム候補）
grep -rA1 "^links:" ./specification/ | grep "links: \[\]"
```

### レビュー報告フォーマット

レビュー結果を以下の形式でユーザーに提示する:

```markdown
## 仕様ドキュメント レビュー報告

### サマリー
- ドキュメント数: N
- 総アイテム数: N
- 問題アイテム数: N

### 問題一覧

#### 🔴 構造的問題（再編が必要）
- `REQ`: 20件がフラット。グループ化未実施 → 分割を推奨
- `OAUTH-REQ` と `AUTH-REQ` が少数ずつ混在 → 集約を推奨
- `NOTIFY-REQ` の親が `REQ` だが、実態は `MESSAGING-REQ` の下が適切 → 階層変更を推奨

#### 🟡 内容的問題（内容の修正が必要）
- `SPEC-003`, `SPEC-007`: 類似した振る舞いを記述している（重複の可能性）
- `REQ-005`: text フィールドが空

#### 🟢 鮮度・管理上の問題（軽微）
- `SPEC-012`: status フィールドが未設定
- `REQ-008`: active: false だがリンクが残っている

### 推奨アクション（優先順）
1. REQ を機能領域別サブドキュメントに分割（→ モード2: 分割）
2. OAUTH-REQ を AUTH-REQ に統合（→ モード2: 集約）
3. NOTIFY-REQ の親を MESSAGING-REQ に変更（→ モード2: 階層変更）
4. SPEC-015 のリンク切れを修正（→ モード3）
```

---

## モード2: 再編（構造の改善）

ユーザーの承認を得てから実行する。**必ず承認を得てから変更を開始すること。**

### 再編操作の種類

| 操作 | 説明 | 使うタイミング |
|---|---|---|
| **分割** | 1つのドキュメント/アイテムを複数に分ける | アイテムが増えすぎた・ドメインが混在している |
| **集約** | 複数のドキュメント/アイテムを1つにまとめる | 細かすぎる分割・ほぼ同一ドメインが分散している |
| **階層変更** | ドキュメントの親子関係を変える | 実態と階層が合っていない・新しい中間ドメインを挿入する |

---

### 操作A: 分割

フラットなドキュメントを機能ドメイン別サブドキュメントに分ける。

#### 計画フォーマット

```markdown
## 分割計画

### 現状
REQ（20件フラット）→ SPEC（18件フラット）

### 提案する新構造
REQ（横断的要件のみ 3件）
├── AUTH-REQ（認証ドメイン 6件）
├── PAY-REQ（決済ドメイン 7件）
└── NTF-REQ（通知ドメイン 4件）

### マイグレーション対象
- REQ-002, REQ-005, REQ-008 → AUTH-REQ へ
- REQ-003, REQ-009, REQ-011 → PAY-REQ へ
```

**⛔ STOP: 計画をユーザーに提示し、承認を得てから実行する。**

#### 実行手順

```bash
# 1. 新サブドキュメントを作成
doorstop create AUTH-REQ ./specification/reqs/auth --parent REQ
# .doorstop.yml の sep: '-' を確認

# 2. アイテムをマイグレーション（1件ずつ）
doorstop add AUTH-REQ   # → AUTH-REQ-001.yml 生成
# 旧アイテムの text 等を新アイテムにコピー
# 旧アイテムを active: false に設定

# 3. SPEC のリンクを張り直す
doorstop link SPEC-005 AUTH-REQ-001
# SPEC-005.yml の links から旧 REQ-002 を削除

# 4. feature タグを更新
# @REQ-002 → @AUTH-REQ-001

# 5. バリデーション
doorstop && spec-weaver audit ./specification/features
```

詳細は `references/reorganization-guide.md` の「パターン1: 分割」を参照。

---

### 操作B: 集約

複数のドキュメントまたはアイテムを1つにまとめる。
ドメインが細かく分かれすぎて管理が煩雑になった場合に使う。

#### ドキュメント集約の計画フォーマット

```markdown
## 集約計画

### 現状
AUTH-REQ（3件）と OAUTH-REQ（2件）が別ドキュメントだが内容が密接に関連

### 提案する新構造
AUTH-REQ（5件：従来の AUTH-REQ + OAUTH-REQ を統合）
OAUTH-REQ ドキュメントは廃止（.doorstop.yml を削除）

### マイグレーション対象
- OAUTH-REQ-001 → AUTH-REQ-004 へ統合
- OAUTH-REQ-002 → AUTH-REQ-005 へ統合
```

**⛔ STOP: 計画をユーザーに提示し、承認を得てから実行する。**

#### 実行手順（ドキュメント集約）

```bash
# 1. 統合先ドキュメントに新アイテムを追加
doorstop add AUTH-REQ   # → AUTH-REQ-004.yml 生成
# OAUTH-REQ-001.yml の内容をコピー
# カスタム属性（status, created_at 等）もコピー

# 2. 統合元アイテムを非アクティブ化
# OAUTH-REQ-001.yml: active: false, migrated_to: AUTH-REQ-004

# 3. SPEC のリンクを張り直す
# 旧 OAUTH-REQ-001 を参照していた SPEC を確認
doorstop
# SPEC-xxx.yml の links を AUTH-REQ-004 に更新

# 4. feature タグを更新
grep -rn "@OAUTH-REQ" ./specification/features/
# @OAUTH-REQ-001 → @AUTH-REQ-004 に置換

# 5. 統合元ドキュメントが空になったら廃止
# .doorstop.yml はそのまま残す（active なアイテムが 0 件になるだけでよい）
# 完全に削除したい場合は doorstop create を解除してディレクトリごと削除
# ただし Git 履歴への影響に注意

# 6. バリデーション
doorstop && spec-weaver audit ./specification/features
```

#### アイテム集約（複数アイテムを1つに統合）

粒度が細かすぎる複数アイテムを1つの包括的なアイテムにまとめる。

```bash
# 1. 統合先アイテムに両方の内容を記述
# SPEC-003.yml に SPEC-003 と SPEC-007 の内容を統合して記述

# 2. 統合元アイテムを非アクティブ化
# SPEC-007.yml: active: false, migrated_to: SPEC-003

# 3. SPEC-007 を参照していたリンクを SPEC-003 に更新
grep -rn "SPEC-007" ./specification/
# 該当ファイルの links を SPEC-003 に変更

# 4. feature タグを更新
grep -rn "@SPEC-007" ./specification/features/
# @SPEC-007 → @SPEC-003 に置換（または削除）
```

詳細は `references/reorganization-guide.md` の「パターン3: 集約」を参照。

---

### 操作C: 階層変更

ドキュメントの親子関係（`--parent`）を変更する。
既存の階層が実態と合わなくなった場合、または新しい中間ドメインを挿入する場合に使う。

#### 計画フォーマット

```markdown
## 階層変更計画

### 現状
REQ
├── AUTH-REQ
└── NOTIFY-REQ   ← 直属だが、実態は MESSAGING の下が適切

### 提案する新構造
REQ
├── AUTH-REQ
└── MESSAGING-REQ（新規）
    └── NOTIFY-REQ（MESSAGING-REQ 配下に移動）

### パターン: 中間ドキュメントの挿入
```

**⛔ STOP: 計画をユーザーに提示し、承認を得てから実行する。**

#### 実行手順（親の変更）

Doorstop の親子関係は `.doorstop.yml` の `parent` フィールドで管理される。
直接編集することで変更できる。

```bash
# 1. 中間ドキュメントが必要な場合は先に作成
doorstop create MESSAGING-REQ ./specification/reqs/messaging --parent REQ

# 2. 移動するドキュメントの .doorstop.yml を編集
# ./specification/reqs/notify/.doorstop.yml を開いて:
# settings:
#   parent: REQ          ← 変更前
#   parent: MESSAGING-REQ  ← 変更後

# 3. 移動後の整合性を確認
doorstop

# 4. 子ドキュメントのリンク先が正しいか確認
# NOTIFY-REQ のアイテムが MESSAGING-REQ の子アイテムにリンクすべきか検討
```

#### 実行手順（サブドキュメントをルートに昇格）

```bash
# ./specification/reqs/auth/.doorstop.yml を編集
# parent: REQ → parent: '' （親なし）に変更
# ただし、REQ のルートドキュメントとの整合性を要確認
doorstop
```

詳細は `references/reorganization-guide.md` の「パターン4: 階層変更」を参照。

---

### Step: 再編後の共通確認

どの操作を行った後も必ず実施する:

```bash
# Doorstop 全体バリデーション
doorstop

# Spec-Weaver 整合性チェック
spec-weaver audit ./specification/features

# ステータス確認
spec-weaver status
```

**⛔ STOP: 全チェックが通過したことを確認し、ユーザーに報告する。**

### コミット

```bash
git add ./specification/
git commit -m "refactor(spec): <操作の概要を記述>"
# 例: refactor(spec): AUTH-REQ と OAUTH-REQ を統合
# 例: refactor(spec): NOTIFY-REQ を MESSAGING-REQ 配下に移動
# 例: refactor(spec): REQ をドメイン別サブドキュメントに分割
```

---

## モード3: 個別アイテムの整合性修正

構造変更は不要だが、個々のアイテムに問題がある場合に使う。

### よくある修正パターン

#### リンク切れの修正

```bash
# 正しい親 REQ-ID にリンクし直す
doorstop link SPEC-015 REQ-010

# 古いリンクを削除（YAML の links フィールドを直接編集）
doorstop edit SPEC-015
```

#### 孤立アイテムへのリンク追加

```bash
# リンクが未設定の SPEC を親 REQ に紐付ける
doorstop link SPEC-007 REQ-003
```

#### 廃止アイテムのクリーンアップ

```bash
# active: false に設定（削除でなく非アクティブ化を推奨）
doorstop edit REQ-008
# → active: false に変更
```

#### 重複アイテムの統合

1. どちらのアイテムを残すか決める
2. 残す方に両方の内容を統合して記述
3. 統合元のアイテムを `active: false` にする
4. 統合先のアイテムに関連する全リンクを確認・更新

---

## チェックリストの詳細

`references/review-checklist.md` を参照。

## 再編の詳細手順

`references/reorganization-guide.md` を参照。
