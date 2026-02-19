#!/usr/bin/env python3
"""コントラスト強調・ノイズ除去・シャープネス調整。

Usage:
    python img_contrast.py <input> [OPTIONS]

Options:
    --factor FLOAT      コントラスト倍率 (1.0=変化なし, 2.0=2倍, default: 1.5)
    --brightness FLOAT  明るさ倍率 (1.0=変化なし, default: 1.0)
    --sharpness FLOAT   シャープネス倍率 (1.0=変化なし, 2.0=シャープ, default: 1.0)
    --denoise           メディアンフィルタでノイズ除去
    --denoise-size INT  ノイズ除去のカーネルサイズ (default: 3, 奇数)
    --auto              自動レベル補正 (ヒストグラム均一化)
    --grayscale         グレースケールに変換してから処理
    -o, --output PATH   出力パス

Examples:
    python img_contrast.py chart.png --factor 1.8
    python img_contrast.py chart.png --auto
    python img_contrast.py chart.png --denoise --factor 1.5 --sharpness 1.5
    python img_contrast.py chart.png --grayscale --auto -o chart_clean.png
"""

from img_utils import base_argparser, load_image, save_image, make_output_path
from PIL import Image, ImageEnhance, ImageFilter, ImageOps


def main():
    parser = base_argparser("コントラスト強調・ノイズ除去")
    parser.add_argument("--factor", type=float, default=1.5, help="コントラスト倍率 (default: 1.5)")
    parser.add_argument("--brightness", type=float, default=1.0, help="明るさ倍率 (default: 1.0)")
    parser.add_argument("--sharpness", type=float, default=1.0, help="シャープネス倍率 (default: 1.0)")
    parser.add_argument("--denoise", action="store_true", help="メディアンフィルタでノイズ除去")
    parser.add_argument("--denoise-size", type=int, default=3, help="ノイズ除去カーネルサイズ (default: 3)")
    parser.add_argument("--auto", action="store_true", help="自動レベル補正")
    parser.add_argument("--grayscale", action="store_true", help="グレースケール変換")
    args = parser.parse_args()

    img = load_image(args.input)
    steps = []

    # グレースケール変換
    if args.grayscale:
        img = img.convert("L")
        steps.append("grayscale")

    # ノイズ除去（コントラスト強調の前に行う）
    if args.denoise:
        img = img.filter(ImageFilter.MedianFilter(size=args.denoise_size))
        steps.append(f"denoise(size={args.denoise_size})")

    # 自動レベル補正
    if args.auto:
        if img.mode == "L":
            img = ImageOps.autocontrast(img, cutoff=1)
        else:
            img = ImageOps.autocontrast(img.convert("RGB"), cutoff=1)
        steps.append("auto-level")

    # 明るさ調整
    if args.brightness != 1.0:
        enhancer = ImageEnhance.Brightness(img)
        img = enhancer.enhance(args.brightness)
        steps.append(f"brightness({args.brightness})")

    # コントラスト強調
    if args.factor != 1.0:
        enhancer = ImageEnhance.Contrast(img)
        img = enhancer.enhance(args.factor)
        steps.append(f"contrast({args.factor})")

    # シャープネス
    if args.sharpness != 1.0:
        enhancer = ImageEnhance.Sharpness(img)
        img = enhancer.enhance(args.sharpness)
        steps.append(f"sharpness({args.sharpness})")

    print(f"🎨 Applied: {' → '.join(steps)}")

    out = make_output_path(args.input, args.output, "_contrast")
    save_image(img, out)


if __name__ == "__main__":
    main()
