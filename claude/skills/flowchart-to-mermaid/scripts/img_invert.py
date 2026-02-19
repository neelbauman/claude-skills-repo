#!/usr/bin/env python3
"""画像の色を反転する。白黒反転やグレースケール反転に対応。

Usage:
    python img_invert.py <input> [OPTIONS]

Options:
    --grayscale     グレースケールに変換してから反転
    -o, --output    出力パス

Examples:
    python img_invert.py dark_chart.png
    python img_invert.py chart.png --grayscale -o chart_inverted.png
"""

from img_utils import base_argparser, load_image, save_image, make_output_path
from PIL import Image, ImageOps


def main():
    parser = base_argparser("画像の色を反転する")
    parser.add_argument("--grayscale", action="store_true", help="グレースケール変換してから反転")
    args = parser.parse_args()

    img = load_image(args.input)
    steps = []

    # グレースケール変換
    if args.grayscale:
        img = img.convert("L")
        steps.append("grayscale")

    # RGBA の場合、アルファチャネルを保持して RGB だけ反転
    if img.mode == "RGBA":
        r, g, b, a = img.split()
        rgb = Image.merge("RGB", (r, g, b))
        inverted_rgb = ImageOps.invert(rgb)
        result = Image.merge("RGBA", (*inverted_rgb.split(), a))
        steps.append("invert (RGB, alpha preserved)")
    elif img.mode == "P":
        # パレットモードは RGB に変換してから反転
        img = img.convert("RGB")
        result = ImageOps.invert(img)
        steps.append("invert (palette→RGB)")
    else:
        result = ImageOps.invert(img)
        steps.append("invert")

    print(f"🔄 Applied: {' → '.join(steps)}")

    out = make_output_path(args.input, args.output, "_inverted")
    save_image(result, out)


if __name__ == "__main__":
    main()
