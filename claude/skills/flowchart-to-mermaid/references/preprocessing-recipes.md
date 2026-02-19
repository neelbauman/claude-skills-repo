# 画像処理スクリプト 詳細リファレンス

各スクリプトの全オプションと、具体的な使用例。
SKILL.md の「使えるツール」表で概要を把握した上で、詳しく知りたいときに参照する。

共通インターフェース: `python scripts/<name>.py <input> -o <output>`
`-o` を省略するとファイル名に自動サフィックスが付く。

---

## img_info.py

画像の基本情報を表示する。前処理の判断材料を得るために最初に実行する。

```bash
python scripts/img_info.py photo.png
```

出力例:
```
📄 File:       photo.png
📐 Size:       1200 x 900 px
🎨 Mode:       RGB
📦 Format:     PNG
💾 File size:  245,320 bytes (239.6 KB)
🔢 Channels:   3 (R, G, B)
📏 Aspect:     4:3
```

800px 未満の場合は拡大を推奨する警告が出る。

---

## img_resize.py

| オプション | 説明 | 例 |
|---|---|---|
| `--scale FLOAT` | 倍率指定 | `--scale 2.0`（2倍拡大） |
| `--width INT` | 幅をpx指定（アスペクト比維持） | `--width 1920` |
| `--height INT` | 高さをpx指定（アスペクト比維持） | `--height 1080` |
| `--max-size INT` | 長辺が指定px以下になるよう縮小 | `--max-size 3000` |
| `--resample STR` | リサンプリング: lanczos, bilinear, bicubic, nearest | `--resample lanczos` |

```bash
# 2倍に拡大（文字が小さいとき、まずこれを試す）
python scripts/img_resize.py input.png --scale 2.0 -o enlarged.png

# 長辺を3000px以下に（大きすぎる画像を扱いやすくする）
python scripts/img_resize.py huge.png --max-size 3000 -o reasonable.png
```

---

## img_crop.py

| オプション | 説明 | 例 |
|---|---|---|
| `--auto` | 余白を自動検出してトリミング | |
| `--auto-threshold INT` | 自動トリミングの閾値 (default: 240) | `--auto-threshold 220` |
| `--box "x1,y1,x2,y2"` | 座標でクロップ範囲を指定 | `--box "100,50,800,600"` |
| `--ratio "x1,y1,x2,y2"` | 0.0-1.0 の比率で範囲指定 | `--ratio "0.0,0.0,0.5,1.0"` |
| `--margin INT` | クロップ後に残す余白 (px) | `--margin 20` |

```bash
# 余白を自動除去（余白多めの画像に）
python scripts/img_crop.py input.png --auto --margin 20 -o trimmed.png

# 左半分を切り出す（2枚の図が横に並んでいるとき）
python scripts/img_crop.py input.png --ratio "0.0,0.0,0.48,1.0" --margin 15 -o left.png
python scripts/img_crop.py input.png --ratio "0.52,0.0,1.0,1.0" --margin 15 -o right.png

# 座標で切り出す（img_info.py のサイズ情報を参考に座標を決める）
python scripts/img_crop.py input.png --box "50,100,600,500" --margin 10 -o region.png
```

**座標の見積もり方:**
img_info.py でサイズを確認し、図の位置を比率で見積もる。
例: 1200x900 の画像で、図が左半分にあるなら box は大体 "0,0,600,900"。
margin を付けておけば多少ずれても切れない。

**自動クロップが図の端を切るとき:**
`--auto-threshold` を下げる（240→220）か、`--margin` を増やす（20→40）。

---

## img_contrast.py

| オプション | 説明 | 例 |
|---|---|---|
| `--factor FLOAT` | コントラスト倍率 (default: 1.5) | `--factor 1.8` |
| `--brightness FLOAT` | 明るさ倍率 (default: 1.0) | `--brightness 1.2` |
| `--sharpness FLOAT` | シャープネス倍率 (default: 1.0) | `--sharpness 1.5` |
| `--denoise` | メディアンフィルタでノイズ除去 | |
| `--denoise-size INT` | ノイズ除去のカーネルサイズ (default: 3) | `--denoise-size 5` |
| `--auto` | 自動レベル補正（ヒストグラム均一化） | |
| `--grayscale` | グレースケールに変換してから処理 | |

オプションは自由に組み合わせられる:

```bash
# 軽めのコントラスト強調（デジタル画像で線が薄いとき）
python scripts/img_contrast.py input.png --factor 1.3 -o enhanced.png

# 写真向けフルセット（ホワイトボード、スキャン等）
python scripts/img_contrast.py photo.png --grayscale --denoise --auto --sharpness 1.5 -o clean.png

# 自動レベル補正だけ（まずこれだけ試すのもあり）
python scripts/img_contrast.py input.png --auto -o auto.png
```

---

## img_erode.py

モルフォロジー演算。線の太さやノイズを操作する。

| オプション | 説明 | 例 |
|---|---|---|
| `--mode STR` | erode, dilate, edge, open, close | `--mode erode` |
| `--iterations INT` | 反復回数 (default: 1) | `--iterations 2` |
| `--kernel INT` | カーネルサイズ (default: 3, 奇数) | `--kernel 5` |
| `--threshold INT` | edge モード時の二値化閾値 | `--threshold 30` |

**モードの違い:**

| モード | 効果 | 使いどころ |
|---|---|---|
| `erode` | 暗い部分（線）を太くする | 線が細くて見づらいとき |
| `dilate` | 明るい部分を広げる（線が細くなる） | 線が太すぎてテキストが潰れているとき |
| `edge` | エッジ（輪郭線）だけを抽出 | 構造を確認したいとき |
| `open` | dilate→erode（小さなノイズ点を消す） | ノイズが多い画像の掃除 |
| `close` | erode→dilate（小さな隙間を埋める） | 途切れた線をつなげたいとき |

```bash
# 線を太くする（最も使用頻度が高い）
python scripts/img_erode.py input.png --mode erode --iterations 1 -o bold.png

# 小さなノイズ点を消す
python scripts/img_erode.py input.png --mode open -o denoised.png

# エッジだけ抽出して構造確認
python scripts/img_erode.py input.png --mode edge -o edges.png
```

---

## img_invert.py

| オプション | 説明 | 例 |
|---|---|---|
| `--grayscale` | グレースケール化してから反転 | |

```bash
# 色を反転（ダークモードのスクショ等を白背景に）
python scripts/img_invert.py dark.png -o light.png

# グレースケール化+反転
python scripts/img_invert.py input.png --grayscale -o inverted_gray.png
```

RGBA 画像の場合はアルファチャネルを保持したまま RGB だけ反転する。

---

## トラブルシューティング

**前処理で線が消えた:** contrast の factor を下げるか、denoise を外す。
細い線をノイズと誤認していることがある。

**自動クロップが図を切った:** `--auto-threshold` を下げるか `--margin` を増やす。
薄い色のノードが端にあると余白と判定されることがある。

**色付きの矢印がグレースケール変換で見えなくなった:** grayscale を使わずカラーのまま
`--auto` で処理するか、invert してから contrast。

**Pillow がインストールされていない:** `pip install Pillow` を実行。
