"""共通ユーティリティ: 出力パス生成、画像読み込み/保存"""

import argparse
from pathlib import Path
from PIL import Image


def make_output_path(input_path: str, output_path: str | None, suffix: str = "_processed") -> Path:
    """出力パスを生成する。-o 未指定なら入力ファイル名に suffix を付加。"""
    if output_path:
        return Path(output_path)
    p = Path(input_path)
    return p.parent / f"{p.stem}{suffix}{p.suffix}"


def load_image(path: str) -> Image.Image:
    """画像を読み込んで返す。"""
    img = Image.open(path)
    img.load()  # lazy loading を強制解決
    return img


def save_image(img: Image.Image, path: Path, quality: int = 95) -> None:
    """画像を保存してパスを表示する。"""
    path.parent.mkdir(parents=True, exist_ok=True)
    save_kwargs = {}
    if path.suffix.lower() in (".jpg", ".jpeg"):
        save_kwargs["quality"] = quality
    img.save(str(path), **save_kwargs)
    print(f"✅ Saved: {path}  ({img.size[0]}x{img.size[1]}, {img.mode})")


def base_argparser(description: str) -> argparse.ArgumentParser:
    """全スクリプト共通の引数パーサーを返す。"""
    parser = argparse.ArgumentParser(description=description)
    parser.add_argument("input", help="入力画像パス")
    parser.add_argument("-o", "--output", default=None, help="出力画像パス (省略時: 自動生成)")
    return parser
