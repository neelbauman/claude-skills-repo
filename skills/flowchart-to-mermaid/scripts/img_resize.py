#!/usr/bin/env python3
"""画像をリサイズする。倍率・幅・高さ指定に対応。

Usage:
    python img_resize.py <input> [OPTIONS]

Options:
    --scale FLOAT       倍率 (例: 2.0 で2倍に拡大)
    --width INT         指定幅にリサイズ (アスペクト比維持)
    --height INT        指定高さにリサイズ (アスペクト比維持)
    --max-size INT      長辺が指定px以下になるようリサイズ
    --resample STR      リサンプリング方式: lanczos(default), bilinear, bicubic, nearest
    -o, --output PATH   出力パス

Examples:
    python img_resize.py chart.png --scale 2.0
    python img_resize.py chart.png --width 1920
    python img_resize.py chart.png --max-size 2000 -o chart_large.png
"""

from img_utils import base_argparser, load_image, save_image, make_output_path
from PIL import Image

RESAMPLE_MAP = {
    "lanczos": Image.LANCZOS,
    "bilinear": Image.BILINEAR,
    "bicubic": Image.BICUBIC,
    "nearest": Image.NEAREST,
}


def main():
    parser = base_argparser("画像をリサイズする")
    parser.add_argument("--scale", type=float, default=None, help="倍率 (例: 2.0)")
    parser.add_argument("--width", type=int, default=None, help="ターゲット幅px")
    parser.add_argument("--height", type=int, default=None, help="ターゲット高さpx")
    parser.add_argument("--max-size", type=int, default=None, help="長辺の最大px")
    parser.add_argument("--resample", default="lanczos", choices=RESAMPLE_MAP.keys(),
                        help="リサンプリング方式 (default: lanczos)")
    args = parser.parse_args()

    img = load_image(args.input)
    w, h = img.size
    resample = RESAMPLE_MAP[args.resample]

    if args.scale:
        new_w, new_h = int(w * args.scale), int(h * args.scale)
    elif args.width:
        ratio = args.width / w
        new_w, new_h = args.width, int(h * ratio)
    elif args.height:
        ratio = args.height / h
        new_w, new_h = int(w * ratio), args.height
    elif args.max_size:
        longest = max(w, h)
        if longest <= args.max_size:
            print(f"ℹ️  画像は既に {args.max_size}px 以下です ({w}x{h})")
            return
        ratio = args.max_size / longest
        new_w, new_h = int(w * ratio), int(h * ratio)
    else:
        parser.error("--scale, --width, --height, --max-size のいずれかを指定してください")
        return

    print(f"🔄 Resize: {w}x{h} → {new_w}x{new_h}")
    resized = img.resize((new_w, new_h), resample)

    out = make_output_path(args.input, args.output, "_resized")
    save_image(resized, out)


if __name__ == "__main__":
    main()
