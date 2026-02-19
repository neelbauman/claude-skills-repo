---
name: git-conventional
description: >
  Conventional Commits 規約に基づいたコミットメッセージを生成する。
  コミット、git commit、コミットメッセージの作成・修正時に使用する。
  変更内容からtype、scope、descriptionを自動判定する。
tags: git, commit, convention
---

# Git Conventional Commits

Conventional Commits 1.0.0 に準拠したコミットメッセージを生成する。

## フォーマット

```
<type>(<scope>): <description>

[optional body]

[optional footer(s)]
```

## Type 一覧

| Type     | 用途                               |
|----------|-----------------------------------|
| feat     | 新機能                             |
| fix      | バグ修正                           |
| docs     | ドキュメントのみの変更              |
| style    | フォーマット変更（コードの意味に影響なし）|
| refactor | リファクタリング（機能追加でもバグ修正でもない）|
| perf     | パフォーマンス改善                  |
| test     | テスト追加・修正                    |
| ci       | CI設定の変更                       |
| chore    | ビルドプロセスや補助ツールの変更     |

## 判定ルール

1. diff の内容から変更の性質を判断する
2. 変更ファイルのパスから scope を推定する
3. description は英語・命令形・50文字以内
4. 破壊的変更がある場合は `!` を付与し、footer に `BREAKING CHANGE:` を記載

## 例

**入力:** src/auth/login.ts にJWTトークン検証を追加

**出力:**
```
feat(auth): add JWT token validation to login flow
```
