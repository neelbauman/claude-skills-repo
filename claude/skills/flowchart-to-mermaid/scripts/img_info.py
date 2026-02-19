#!/usr/bin/env python3
"""画像の基本情報（サイズ、モード、フォーマット等）を表示する。

Usage:
    python img_info.py <input_image>

Example:
    python img_info.py flowchart.png
"""

import argparse
import os
from pathlib import Path
from PIL import Image


def main():
    parser = argparse.ArgumentParser(description="画像の基本情報を表示する")
    parser.add_argument("input", help="入力画像パス")
    args = parser.parse_args()

    path = Path(args.input)
    if not path.exists():
        print(f"❌ File not found: {path}")
        raise SystemExit(1)

    img = Image.open(path)
    file_size = os.path.getsize(path)

    print(f"📄 File:       {path}")
    print(f"📐 Size:       {img.size[0]} x {img.size[1]} px")
    print(f"🎨 Mode:       {img.mode}")
    print(f"📦 Format:     {img.format or 'N/A'}")
    print(f"💾 File size:  {file_size:,} bytes ({file_size / 1024:.1f} KB)")

    if img.info.get("dpi"):
        dpi = img.info["dpi"]
        print(f"🔍 DPI:        {dpi[0]} x {dpi[1]}")

    # 色のチャネル情報
    if img.mode in ("RGB", "RGBA"):
        print(f"🔢 Channels:   {len(img.getbands())} ({', '.join(img.getbands())})")
    elif img.mode == "L":
        print(f"🔢 Channels:   1 (Grayscale)")

    # 画像が小さい場合の警告
    w, h = img.size
    if w < 400 or h < 400:
        print(f"⚠️  小さい画像です。img_resize.py --scale 2.0 で拡大を推奨します。")

    # アスペクト比
    from math import gcd
    g = gcd(w, h)
    print(f"📏 Aspect:     {w // g}:{h // g}")


if __name__ == "__main__":
    main()
