#!/usr/bin/env python3
"""画像をクロップ（切り抜き）する。

Usage:
    python img_crop.py <input> [OPTIONS]

Options:
    --box "x1,y1,x2,y2"   クロップ範囲 (左上x, 左上y, 右下x, 右下y)
    --margin INT           全辺に余白を残してクロップ (px)
    --ratio "x1,y1,x2,y2" 0.0-1.0 の比率で範囲指定
    --auto                 余白を自動検出してトリミング
    --auto-threshold INT   自動トリミングの閾値 (default: 240, 白に近い色を余白と判定)
    -o, --output PATH      出力パス

Examples:
    python img_crop.py chart.png --box "100,50,800,600"
    python img_crop.py chart.png --ratio "0.1,0.1,0.9,0.9"
    python img_crop.py chart.png --auto
    python img_crop.py chart.png --margin 20 --box "100,50,800,600"
"""

from img_utils import base_argparser, load_image, save_image, make_output_path
from PIL import Image, ImageOps


def auto_crop(img: Image.Image, threshold: int = 240) -> tuple[int, int, int, int]:
    """白い余白を自動検出してクロップ範囲を返す。"""
    gray = img.convert("L")
    # threshold以下（暗い部分）をコンテンツとみなす
    bbox = gray.point(lambda p: 0 if p > threshold else 255).getbbox()
    if bbox is None:
        return (0, 0, img.size[0], img.size[1])
    return bbox


def main():
    parser = base_argparser("画像をクロップする")
    parser.add_argument("--box", default=None, help='クロップ範囲 "x1,y1,x2,y2"')
    parser.add_argument("--ratio", default=None, help='比率で範囲指定 "x1,y1,x2,y2" (0.0-1.0)')
    parser.add_argument("--margin", type=int, default=0, help="クロップ後に残す余白 (px)")
    parser.add_argument("--auto", action="store_true", help="余白を自動検出してトリミング")
    parser.add_argument("--auto-threshold", type=int, default=240, help="自動トリミング閾値 (default: 240)")
    args = parser.parse_args()

    img = load_image(args.input)
    w, h = img.size

    if args.auto:
        box = auto_crop(img, args.auto_threshold)
        print(f"🔍 Auto-detected content area: {box}")
    elif args.ratio:
        parts = [float(x.strip()) for x in args.ratio.split(",")]
        if len(parts) != 4:
            parser.error("--ratio は x1,y1,x2,y2 形式で4値を指定")
            return
        box = (int(parts[0] * w), int(parts[1] * h), int(parts[2] * w), int(parts[3] * h))
    elif args.box:
        parts = [int(x.strip()) for x in args.box.split(",")]
        if len(parts) != 4:
            parser.error("--box は x1,y1,x2,y2 形式で4値を指定")
            return
        box = tuple(parts)
    else:
        parser.error("--box, --ratio, --auto のいずれかを指定してください")
        return

    # margin 適用
    if args.margin:
        m = args.margin
        box = (max(0, box[0] - m), max(0, box[1] - m),
               min(w, box[2] + m), min(h, box[3] + m))

    print(f"✂️  Crop: ({box[0]}, {box[1]}) → ({box[2]}, {box[3]})")
    print(f"   Result size: {box[2] - box[0]}x{box[3] - box[1]} px")
    cropped = img.crop(box)

    out = make_output_path(args.input, args.output, "_cropped")
    save_image(cropped, out)


if __name__ == "__main__":
    main()
