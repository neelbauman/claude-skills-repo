---
name: doorstop-review
description: >
  Doorstop + Gherkin 仕様管理のドキュメントレビューと再編スキル。
  増えたドキュメントが整合性をもって記述・配置されているかをチェックし、
  必要があれば再整理を提案・実行する。
  ユーザーが「仕様を見直したい」「ドキュメントが増えてきた」「構造を整理したい」
  「仕様のレビュー」「再編」「整合性チェック」「ドキュメント整理」を話題にした場合は
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
- `REQ`: 20件のアイテムがフラットに並んでいる（グループ化が未実施）
- `SPEC-015`: リンク先 REQ-999 が存在しない（リンク切れ）

#### 🟡 内容的問題（内容の修正が必要）
- `SPEC-003`, `SPEC-007`: 類似した振る舞いを記述している（重複の可能性）
- `REQ-005`: text フィールドが空

#### 🟢 鮮度・管理上の問題（軽微）
- `SPEC-012`: status フィールドが未設定
- `REQ-008`: active: false だがリンクが残っている

### 推奨アクション
1. REQ を機能領域別サブドキュメントに分割（→ モード2）
2. SPEC-015 のリンク切れを修正
3. SPEC-003 と SPEC-007 を比較・統合を検討
```

---

## モード2: 再編（構造の改善）

ユーザーの承認を得てから実行する。**必ず承認を得てから変更を開始すること。**

### Step 1: 再編計画の策定

レビュー結果をもとに、以下を決定してユーザーに提示する:

```markdown
## 再編計画

### 現状
REQ（20件フラット）→ SPEC（18件フラット）→ features/

### 提案する新構造
REQ（横断的要件のみ 3件）
├── AUTH-REQ（認証ドメイン 6件）
├── PAY-REQ（決済ドメイン 7件）
└── NTF-REQ（通知ドメイン 4件）

SPEC（横断的仕様のみ）
├── AUTH（認証仕様）
├── PAY（決済仕様）
└── NTF（通知仕様）

### マイグレーション対象
- REQ-002, REQ-005, REQ-008 → AUTH-REQ へ移行
- REQ-003, REQ-009, REQ-011 → PAY-REQ へ移行
（以下略）

### 影響範囲
- SPEC のリンク先 REQ-ID が変わるため、doorstop link を再設定
- feature タグ @REQ-002 → @AUTH-REQ-001 に更新
```

**⛔ STOP: 計画をユーザーに提示し、承認を得てから Step 2 へ進む。**

### Step 2: サブドキュメントの作成

```bash
# 新しいドメインドキュメントを作成
doorstop create AUTH-REQ ./specification/reqs/auth --parent REQ
doorstop create PAY-REQ  ./specification/reqs/payment --parent REQ

doorstop create AUTH ./specification/specs/auth --parent AUTH-REQ
doorstop create PAY  ./specification/specs/payment --parent PAY-REQ

# .doorstop.yml の sep を確認・修正（Spec-Weaver 対応）
# 各ディレクトリの .doorstop.yml を開いて sep: '-' を設定
```

### Step 3: アイテムのマイグレーション

Doorstop はアイテムの「移動」コマンドを持たないため、以下の手順で移行する:

```
1. 新ドキュメントに新アイテムを追加（doorstop add）
2. 旧アイテムの text を新アイテムの text にコピー
3. 旧アイテムへのリンクを持つ SPEC を新アイテムへリンク張り直し
4. 旧アイテムを active: false に設定（削除でなく非アクティブ化）
5. doorstop でバリデーション
```

詳細手順は `references/reorganization-guide.md` を参照。

### Step 4: feature タグの更新

feature ファイルのシナリオタグを新 ID に更新する:

```bash
# 変更前: @REQ-002 → 変更後: @AUTH-REQ-001 の例
# feature ファイル内の旧タグを検索して確認
grep -rn "@REQ-002" ./specification/features/

# 該当 feature ファイルを編集してタグを更新
```

### Step 5: 整合性の再確認

```bash
# Doorstop 全体バリデーション
doorstop

# Spec-Weaver 整合性チェック
spec-weaver audit ./specification/features

# ステータス確認
spec-weaver status
```

**⛔ STOP: 全チェックが通過したことを確認し、ユーザーに報告する。**

### Step 6: コミット

```bash
# 再編内容をコミット
git add ./specification/
git commit -m "refactor(spec): ドキュメント構造を機能ドメイン別に再編"
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
