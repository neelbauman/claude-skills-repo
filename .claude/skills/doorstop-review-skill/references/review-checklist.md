# レビューチェックリスト

Doorstop + Gherkin ドキュメントをレビューする際の確認項目一覧。
モード1（レビュー）での診断時に、このリストを順に実施する。

---

## A. 構造チェック

### A-1. ドキュメント一覧の把握

```bash
find ./specification -name ".doorstop.yml" | sort
```

確認すること:
- [ ] どのドキュメント（prefix）が存在するか
- [ ] 各ドキュメントの親子関係が意図通りか
- [ ] 孤立したディレクトリ（`.doorstop.yml` が missing）がないか

### A-2. アイテム数の確認

```bash
# 各ドキュメントのアイテム数をカウント
for dir in $(find ./specification -name ".doorstop.yml" -exec dirname {} \;); do
  count=$(find "$dir" -maxdepth 1 -name "*.yml" ! -name ".doorstop.yml" | wc -l)
  echo "$count  $dir"
done | sort -rn
```

判断基準:
- [ ] 1ドキュメントに **10件以上** → サブドキュメント分割を推奨
- [ ] 1ドキュメントに **20件以上** → 分割が必要（見通しが著しく悪い）
- [ ] 1ドキュメントに **0件** → 空ドキュメント（削除候補）

### A-3. `level` フィールドの利用状況

```bash
# level が設定されているアイテムを確認
grep -rh "^level:" ./specification/reqs ./specification/specs | sort | uniq -c
```

確認すること:
- [ ] `level: 1.0` が複数ある → セクション構造を使っているか確認
- [ ] `normative: false` の `level: X.0` アイテムがセクション見出しとして機能しているか
- [ ] level が連番になっていて飛びがないか（飛びは削除済みアイテムを示す）

---

## B. リンク整合性チェック

### B-1. Doorstop バリデーション

```bash
doorstop
```

確認すること:
- [ ] エラー・警告が 0 件
- [ ] "WARNING" メッセージの内容を把握（未レビュー・リンク切れ等）

### B-2. 孤立アイテムの検出

```bash
# links: [] のアイテムを列挙
grep -rl "^links: \[\]" ./specification/specs/ 2>/dev/null
grep -rl "^links: \[\]" ./specification/reqs/ 2>/dev/null
```

確認すること:
- [ ] SPEC で `links: []` → 親 REQ へのリンクが未設定（要修正）
- [ ] REQ で `links: []` → ルートドキュメントなら正常。子ドキュメントなら要確認
- [ ] 拡張ドキュメント（DESIGN/PLAN/ADR）で `links: []` → SPEC へのリンクが未設定

### B-3. active: false のアイテムへのリンク

```bash
# 非アクティブアイテムのリストを作成
inactive=$(grep -rl "^active: false" ./specification/ | xargs grep -h "^  prefix:" 2>/dev/null)

# 非アクティブ ID を他アイテムが参照していないか確認
# （手動確認: doorstop バリデーションで検出されることが多い）
doorstop
```

- [ ] 非アクティブアイテムへのリンクが残っていないか

---

## C. 内容チェック

### C-1. text フィールドの充足

```bash
# text が空またはプレースホルダーのままのアイテム
grep -rA3 "^text:" ./specification/reqs ./specification/specs | \
  grep -B1 "text: ''" | grep "\.yml"
```

確認すること:
- [ ] `text: ''` のアイテム → 記述が未完了
- [ ] text の内容が 1行以下 → 要件・仕様として不十分な可能性

### C-2. status フィールドの管理

```bash
# status が設定されていないアイテム
find ./specification/reqs ./specification/specs -name "*.yml" ! -name ".doorstop.yml" | \
  xargs grep -L "^status:" 2>/dev/null
```

確認すること:
- [ ] `status` 未設定のアイテムが多数 → 管理されていない状態（一括更新を推奨）
- [ ] `status: deprecated` なのに `active: true` → `active: false` に変更すべき
- [ ] `status: implemented` で `active: true` は正常

### C-3. 重複・類似アイテムの検出

自動的な重複検出は困難なため、以下の観点で目視確認する:

```bash
# header フィールドの一覧（類似タイトルを探す）
grep -rh "^header:" ./specification/reqs ./specification/specs | sort

# text の最初の行（概要）を抽出して並べる
grep -rA1 "^text: |" ./specification/reqs ./specification/specs | \
  grep -v "^text:" | grep -v "^--" | head -40
```

確認すること:
- [ ] 同じ機能を別の言葉で説明しているアイテムがないか
- [ ] 分割すべき複合要件がないか（1アイテムに複数の要件が混在）
- [ ] 統合すべき粒度の細かすぎるアイテムがないか

---

## D. Gherkin 連携チェック

### D-1. Spec-Weaver 監査

```bash
spec-weaver audit ./specification/features
```

確認すること:
- [ ] エラーが 0 件
- [ ] `MISSING_SPEC` → feature のタグに対応する SPEC が存在しない
- [ ] `UNTESTED_SPEC` → SPEC に対応する feature シナリオがない（`testable: true` のもの）

### D-2. testable 設定の確認

```bash
# testable: false が設定されているアイテム
grep -rl "^testable: false" ./specification/specs/ 2>/dev/null

# testable 未設定（デフォルト: testable とみなされる）のアイテム数
find ./specification/specs -name "*.yml" ! -name ".doorstop.yml" | \
  xargs grep -L "^testable:" 2>/dev/null | wc -l
```

確認すること:
- [ ] Gherkin でテストできない仕様（UI見た目、設定値等）に `testable: false` が付いているか
- [ ] `testable: false` が多すぎる → 仕様がテスト可能な形で書かれていない可能性

### D-3. feature ファイルのタグ確認

```bash
# feature ファイルで使われているタグの一覧
grep -rh "@[A-Z][A-Z0-9-]*" ./specification/features/ | \
  grep -o "@[A-Z][A-Z0-9-]*" | sort | uniq
```

確認すること:
- [ ] タグに対応する SPEC/REQ ID が実際に存在するか
- [ ] 古い ID（リネーム後の旧 ID 等）のタグが残っていないか
- [ ] タグなしのシナリオがないか（全シナリオに SPEC タグを付けるべき）

---

## E. 鮮度チェック

### E-1. 長期間更新されていないアイテム

```bash
# spec-weaver で stale チェック（90日以上更新なし）
spec-weaver audit ./specification/features --stale-days 90
```

確認すること:
- [ ] stale として検出されたアイテムは内容が今も有効か
- [ ] 実装が変わったのに SPEC が更新されていないケースはないか

### E-2. deprecated アイテムの整理

```bash
# deprecated アイテム一覧
grep -rl "status: deprecated" ./specification/ 2>/dev/null
```

確認すること:
- [ ] `status: deprecated` かつ `active: true` → `active: false` への移行を検討
- [ ] 参照されている deprecated アイテムがないか（リンク先として残っている場合）

---

## レビュー報告のまとめ方

上記チェックの結果を以下の重要度で分類して報告する:

| 重要度 | 基準 | 対応 |
|---|---|---|
| 🔴 構造的問題 | ドキュメント構造に起因する問題（分割・統合が必要） | モード2で再編 |
| 🟡 内容的問題 | アイテムの記述・リンクに問題がある | モード3で個別修正 |
| 🟢 管理上の問題 | status 未設定・stale 等の軽微な問題 | 随時修正 |
