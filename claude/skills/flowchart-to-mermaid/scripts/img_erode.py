#!/usr/bin/env python3
"""モルフォロジー演算でエッジ（線）を強調する。

erode（収縮）で暗い領域を広げて線を太くする。
dilate（膨張）で明るい領域を広げてノイズを除去する。
edge モードでは膨張と収縮の差分からエッジを抽出する。

Usage:
    python img_erode.py <input> [OPTIONS]

Options:
    --mode STR          演算モード: erode(default), dilate, edge, open, close
    --iterations INT    反復回数 (default: 1)
    --kernel INT        カーネルサイズ (default: 3, 奇数)
    --threshold INT     エッジモード時の二値化閾値 (default: 0, 0=閾値なし)
    -o, --output PATH   出力パス

Modes:
    erode   線を太くする（暗い部分を膨張）← フローチャートの線強調に最適
    dilate  線を細くする（明るい部分を膨張）
    edge    エッジ抽出（膨張-収縮）
    open    オープニング（dilate→erode, 小さなノイズ除去）
    close   クロージング（erode→dilate, 小さな隙間を埋める）

Examples:
    python img_erode.py chart.png --mode erode --iterations 1
    python img_erode.py chart.png --mode edge --kernel 3
    python img_erode.py chart.png --mode close --iterations 2 -o chart_clean.png
"""

from img_utils import base_argparser, load_image, save_image, make_output_path
from PIL import Image, ImageFilter


def erode(img: Image.Image, kernel: int = 3, iterations: int = 1) -> Image.Image:
    """収縮: 暗い領域を広げる (= 線を太くする)"""
    result = img
    for _ in range(iterations):
        result = result.filter(ImageFilter.MinFilter(kernel))
    return result


def dilate(img: Image.Image, kernel: int = 3, iterations: int = 1) -> Image.Image:
    """膨張: 明るい領域を広げる (= 線を細くする)"""
    result = img
    for _ in range(iterations):
        result = result.filter(ImageFilter.MaxFilter(kernel))
    return result


def edge_detect(img: Image.Image, kernel: int = 3) -> Image.Image:
    """エッジ抽出: 膨張 - 収縮 の差分"""
    from PIL import ImageChops
    dilated = dilate(img, kernel, 1)
    eroded = erode(img, kernel, 1)
    return ImageChops.difference(dilated, eroded)


def opening(img: Image.Image, kernel: int = 3, iterations: int = 1) -> Image.Image:
    """オープニング: 膨張→収縮 (小さなノイズ除去)"""
    return erode(dilate(img, kernel, iterations), kernel, iterations)


def closing(img: Image.Image, kernel: int = 3, iterations: int = 1) -> Image.Image:
    """クロージング: 収縮→膨張 (小さな隙間を埋める)"""
    return dilate(erode(img, kernel, iterations), kernel, iterations)


def main():
    parser = base_argparser("モルフォロジー演算でエッジを強調する")
    parser.add_argument("--mode", default="erode",
                        choices=["erode", "dilate", "edge", "open", "close"],
                        help="演算モード (default: erode)")
    parser.add_argument("--iterations", type=int, default=1, help="反復回数 (default: 1)")
    parser.add_argument("--kernel", type=int, default=3, help="カーネルサイズ (default: 3, 奇数)")
    parser.add_argument("--threshold", type=int, default=0,
                        help="エッジモード時の二値化閾値 (default: 0=なし)")
    args = parser.parse_args()

    if args.kernel % 2 == 0:
        parser.error("--kernel は奇数を指定してください (3, 5, 7, ...)")
        return

    img = load_image(args.input)

    # グレースケールに変換して処理（カラー画像はチャネル別に処理）
    if img.mode in ("RGB", "RGBA"):
        channels = img.split()
        alpha = None
        if img.mode == "RGBA":
            alpha = channels[3]
            channels = channels[:3]

        ops = {"erode": erode, "dilate": dilate, "edge": edge_detect,
               "open": opening, "close": closing}

        if args.mode == "edge":
            processed = [ops[args.mode](ch, args.kernel) for ch in channels]
        else:
            processed = [ops[args.mode](ch, args.kernel, args.iterations) for ch in channels]

        if alpha:
            result = Image.merge("RGBA", (*processed, alpha))
        else:
            result = Image.merge("RGB", tuple(processed))
    else:
        if args.mode == "erode":
            result = erode(img, args.kernel, args.iterations)
        elif args.mode == "dilate":
            result = dilate(img, args.kernel, args.iterations)
        elif args.mode == "edge":
            result = edge_detect(img, args.kernel)
        elif args.mode == "open":
            result = opening(img, args.kernel, args.iterations)
        elif args.mode == "close":
            result = closing(img, args.kernel, args.iterations)

    # エッジモードでの二値化
    if args.mode == "edge" and args.threshold > 0:
        result = result.point(lambda p: 255 if p > args.threshold else 0)

    print(f"🔲 Applied: {args.mode} (kernel={args.kernel}, iterations={args.iterations})")

    out = make_output_path(args.input, args.output, f"_{args.mode}")
    save_image(result, out)


if __name__ == "__main__":
    main()
