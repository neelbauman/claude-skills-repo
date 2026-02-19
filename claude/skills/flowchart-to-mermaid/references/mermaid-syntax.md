# Mermaid フローチャート構文リファレンス

フローチャート画像から Mermaid コードを生成する際の構文ガイド。

## 目次

1. [ダイアグラムタイプの選択](#ダイアグラムタイプの選択)
2. [フローチャート構文](#フローチャート構文)
3. [ノード形状](#ノード形状)
4. [接続（エッジ）](#接続エッジ)
5. [サブグラフ](#サブグラフ)
6. [スタイリング](#スタイリング)
7. [シーケンス図](#シーケンス図)
8. [状態遷移図](#状態遷移図)
9. [よくあるパターン](#よくあるパターン)

---

## ダイアグラムタイプの選択

| 元画像の特徴 | Mermaid タイプ |
|---|---|
| 箱と矢印のフロー図 | `flowchart TD` or `LR` |
| 時系列のやりとり（メッセージング） | `sequenceDiagram` |
| 状態と遷移の図 | `stateDiagram-v2` |

## フローチャート構文

### 方向

```mermaid
flowchart TD   %% Top → Down (上から下)
flowchart LR   %% Left → Right (左から右)
flowchart BT   %% Bottom → Top (下から上)
flowchart RL   %% Right → Left (右から左)
```

### ノード形状

```mermaid
flowchart TD
    A[長方形]
    B(角丸長方形)
    C([スタジアム型])
    D[[サブルーチン]]
    E[(データベース)]
    F((円))
    G{ひし形}
    H{/平行四辺形/}
    I[\逆平行四辺形\]
    J[/台形\]
    K[\逆台形/]
    L>非対称]
    M(((二重円)))
```

### 形状と意味の対応

| 形状 | 構文 | 意味 |
|---|---|---|
| 長方形 | `[text]` | 処理・アクション |
| ひし形 | `{text}` | 判断・分岐 |
| 角丸 | `(text)` | 汎用ノード |
| スタジアム | `([text])` | 開始・終了 |
| 平行四辺形 | `[/text/]` | 入出力 |
| データベース | `[(text)]` | データストア |
| 円 | `((text))` | 接続子 |
| サブルーチン | `[[text]]` | サブプロセス |

## 接続（エッジ）

### 基本

```mermaid
flowchart LR
    A --> B           %% 矢印
    C --- D           %% 線のみ
    E -.- F           %% 破線
    G -.-> H          %% 破線矢印
    I ==> J           %% 太線矢印
    K <--> L          %% 双方向矢印
```

### ラベル付き

```mermaid
flowchart LR
    A -->|Yes| B
    C -->|No| D
    E ---|ラベル| F
    G -.->|optional| H
```

### 接続の長さ（特殊）

```
--->     少し長い
---->    もっと長い
```

## サブグラフ

```mermaid
flowchart TD
    subgraph sg1[グループ名]
        A --> B
    end
    subgraph sg2[別グループ]
        C --> D
    end
    B --> C
```

サブグラフの入れ子も可能:

```mermaid
flowchart TD
    subgraph outer[外側]
        subgraph inner[内側]
            A --> B
        end
        B --> C
    end
```

## スタイリング

```mermaid
flowchart TD
    A[重要] --> B[通常]
    style A fill:#f96,stroke:#333,stroke-width:2px
    style B fill:#bbf,stroke:#333
```

クラス定義:

```mermaid
flowchart TD
    A:::important --> B
    classDef important fill:#f96,stroke:#333,stroke-width:2px
```

## シーケンス図

```mermaid
sequenceDiagram
    participant A as ユーザー
    participant B as サーバー
    A->>B: リクエスト
    B-->>A: レスポンス
    Note over A,B: 注釈
    alt 成功
        B->>A: 200 OK
    else 失敗
        B->>A: 500 Error
    end
```

## 状態遷移図

```mermaid
stateDiagram-v2
    [*] --> Idle
    Idle --> Processing: start
    Processing --> Done: complete
    Processing --> Error: fail
    Error --> Idle: retry
    Done --> [*]
```

## よくあるパターン

### 分岐と合流

```mermaid
flowchart TD
    start([開始]) --> check{条件?}
    check -->|Yes| pathA[処理A]
    check -->|No| pathB[処理B]
    pathA --> merge[合流]
    pathB --> merge
    merge --> finish([終了])
```

### ループ

```mermaid
flowchart TD
    init[初期化] --> loop{条件?}
    loop -->|Yes| process[処理]
    process --> loop
    loop -->|No| done([終了])
```

### エラーハンドリング

```mermaid
flowchart TD
    process[処理] --> check{成功?}
    check -->|Yes| next[次へ]
    check -->|No| error[エラー処理]
    error --> retry{リトライ?}
    retry -->|Yes| process
    retry -->|No| fail([失敗終了])
```

### 並列処理

```mermaid
flowchart TD
    start([開始]) --> fork
    fork --> taskA[タスクA]
    fork --> taskB[タスクB]
    fork --> taskC[タスクC]
    taskA --> join
    taskB --> join
    taskC --> join
    join --> finish([終了])
```
