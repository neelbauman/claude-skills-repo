# semantic-review-skill

## トリガー条件

以下のいずれかに該当する場合、このスキルを適用すること:

- `spec-weaverセマンティックレビュー` というフレーズを含む（CLIからの自動呼び出し）
- `セマンティックレビュー` または `semantic review`
- `仕様と実装の乖離` を確認したい
- `--file` オプションで仕様YAML・featureファイル・実装コードが渡されている

## 役割

渡されたファイル群（仕様YAML・Gherkin feature・ステップ定義・実装コード）を解析し、
以下の3観点で**意味的な整合性**を評価すること。

## 評価観点

### 1. missing_implementation（実装漏れ）
仕様書に記述された要件・振る舞いが、実装コードに存在しないか不完全である。

### 2. undocumented_feature（仕様欠落）
実装コードに存在する振る舞い・機能が、仕様書に記述されていない。

### 3. semantic_contradiction（意味的矛盾）
仕様書とコードの両方に記述はあるが、内容が食い違っている
（例: エラー時の挙動、返り値の型、副作用の有無）。

## severity の基準

| レベル | 対象 |
|--------|------|
| **high** | セキュリティ・データ整合性・主要機能に関わる乖離 |
| **medium** | エラーハンドリング・境界値・副作用の不一致 |
| **low** | 軽微な表現の違い・補足情報の欠落 |

## 出力モード

### CLIモード（自動判定）

プロンプトに「純粋なJSONのみで出力」という指示が含まれている場合は、
**説明文・コードブロックなしで、JSONオブジェクトのみを出力すること**。

```json
{
  "schema_version": "1.0",
  "item_id": "SPEC-003",
  "item_title": "audit コマンド",
  "reviewed_files": [
    "specification/specs/SPEC-003.yml",
    "specification/features/audit.feature",
    "src/spec_weaver/cli.py"
  ],
  "findings": [
    {
      "kind": "missing_implementation",
      "severity": "medium",
      "title": "--prefix オプションの動作が未検証",
      "detail": "仕様書には --prefix オプションで監査対象を絞れると記載されているが、Gherkin シナリオにそのケースが存在しない",
      "location": "specification/specs/SPEC-003.yml §入力オプション"
    }
  ],
  "summary": "全体的に仕様と実装は整合しているが、--prefix オプションのテストカバレッジに軽微な欠落がある。"
}
```

findingが存在しない場合は `"findings": []` とすること。

### インタラクティブモード（ユーザー直接依頼）

ユーザーが会話の中で直接依頼した場合は、Markdown形式で報告すること:

```markdown
## セマンティックレビュー結果: SPEC-003 — audit コマンド

### レビュー対象ファイル
- specification/specs/SPEC-003.yml
- specification/features/audit.feature
- src/spec_weaver/cli.py

### Findings

| 重大度 | 種別 | タイトル |
|--------|------|---------|
| medium | missing_implementation | --prefix オプションの動作が未検証 |

#### [medium] --prefix オプションの動作が未検証
仕様書には --prefix オプションで監査対象を絞れると記載されているが、
Gherkin シナリオにそのケースが存在しない。

**場所**: specification/specs/SPEC-003.yml §入力オプション

### 総評
全体的に仕様と実装は整合しているが、--prefix オプションのテストカバレッジに軽微な欠落がある。
```

## 注意事項

- findingは実際に確認できる乖離のみを報告すること（推測で報告しない）
- ファイルが提供されていない観点（例: 実装ファイルなし）については「レビュー不能」として明記すること
- Gherkin シナリオはあくまで「期待される振る舞い」の仕様であり、実装との乖離は finding として報告してよい
