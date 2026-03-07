#!/usr/bin/env python3
"""Doorstopプロジェクトのバリデーションとトレーサビリティレポート生成。

REQ → SPEC → IMPL/TST の4階層対応。
構造検証、リンク整合性、ref存在チェック、機能グループ別カバレッジ、
レビュー状態の追跡を行い、HTMLトレーサビリティレポートを出力する。

Usage:
    python validate_and_report.py <project-dir> [--output-dir ./reports] [--strict] [--json]
"""

import argparse
import html
import json
import os
import sys
from collections import defaultdict
from datetime import datetime

try:
    import doorstop
except ImportError:
    print("ERROR: doorstop がインストールされていません。", file=sys.stderr)
    sys.exit(1)


def get_group(item):
    try:
        g = item.get("group")
        return g if g else "(未分類)"
    except (AttributeError, KeyError):
        return "(未分類)"


def get_ref(item):
    try:
        return item.ref or ""
    except (AttributeError, KeyError):
        return ""


def validate_tree(tree, strict=False, project_dir="."):
    """ドキュメントツリーを検証する。"""
    issues = {"errors": [], "warnings": [], "info": []}

    # 基本チェック
    for document in tree:
        for item in document:
            if not item.text.strip():
                issues["warnings"].append(f"{item.uid}: テキストが空です")
            if not item.active:
                issues["warnings"].append(f"{item.uid}: 非アクティブです")

    # 親子リンクの検証
    docs = {doc.prefix: doc for doc in tree}
    for document in tree:
        if not document.parent:
            continue
        parent_doc = docs.get(document.parent)
        if parent_doc is None:
            issues["errors"].append(
                f"{document.prefix}: 親ドキュメント '{document.parent}' が見つかりません"
            )
            continue

        parent_uids = {str(item.uid) for item in parent_doc}
        for item in document:
            linked_parents = [
                str(link) for link in item.links
                if str(link).startswith(document.parent)
            ]
            if not linked_parents:
                issues["warnings"].append(
                    f"{item.uid} [{get_group(item)}]: "
                    f"親ドキュメント {document.parent} へのリンクがありません"
                )
            for link in linked_parents:
                link_uid = link.split(":")[0] if ":" in link else link
                if link_uid not in parent_uids:
                    issues["errors"].append(
                        f"{item.uid}: リンク先 {link_uid} が存在しません"
                    )

        # クロスグループリンク警告
        parent_groups = {str(i.uid): get_group(i) for i in parent_doc}
        for item in document:
            child_group = get_group(item)
            if child_group == "(未分類)":
                continue
            for link in item.links:
                link_str = str(link)
                if link_str in parent_groups:
                    pg = parent_groups[link_str]
                    if pg != "(未分類)" and pg != child_group:
                        issues["warnings"].append(
                            f"{item.uid} [{child_group}] → {link_str} [{pg}]: "
                            f"クロスグループリンクです"
                        )

        # 逆方向チェック（strictモード）
        if strict:
            child_links = defaultdict(set)
            for item in document:
                for link in item.links:
                    link_str = str(link)
                    if link_str.startswith(document.parent):
                        child_links[link_str].add(str(item.uid))
            for parent_item in parent_doc:
                if str(parent_item.uid) not in child_links:
                    issues["warnings"].append(
                        f"{parent_item.uid} [{get_group(parent_item)}]: "
                        f"子ドキュメント {document.prefix} からのリンクがありません"
                    )

    # ref存在チェック（IMPL, TST）
    ref_docs = {"IMPL", "TST"}
    for document in tree:
        if document.prefix not in ref_docs:
            continue
        for item in document:
            ref = get_ref(item)
            if not ref:
                continue
            # ファイルパス部分を抽出（:: 以降はクラス/関数名）
            filepath = ref.split("::")[0]
            full_path = os.path.join(project_dir, filepath)
            if not os.path.exists(full_path):
                issues["warnings"].append(
                    f"{item.uid}: ref '{ref}' のファイルが存在しません"
                )

    # レビュー状態チェック
    unreviewed = []
    for document in tree:
        for item in document:
            if item.reviewed is None or item.reviewed == "":
                unreviewed.append(str(item.uid))
    if unreviewed:
        issues["info"].append(
            f"未レビューアイテム: {len(unreviewed)}件 "
            f"({', '.join(unreviewed[:10])}{'...' if len(unreviewed) > 10 else ''})"
        )

    return issues


def build_traceability_matrix(tree):
    """トレーサビリティマトリクスを構築する。

    REQ → SPEC → IMPL/TST の構造で、SPECに2つの子がある場合も対応。
    """
    docs = list(tree)
    prefixes = [d.prefix for d in docs]
    matrix = []

    root_docs = [d for d in docs if not d.parent]
    for root_doc in root_docs:
        for item in root_doc:
            row = {root_doc.prefix: item, "_group": get_group(item)}
            matrix.append(row)

    def expand_children(doc, parent_prefix):
        child_docs = [d for d in docs if d.parent == doc.prefix]
        for child_doc in child_docs:
            link_map = defaultdict(list)
            for child_item in child_doc:
                for link in child_item.links:
                    link_str = str(link)
                    if link_str.startswith(parent_prefix):
                        link_map[link_str].append(child_item)

            new_rows = []
            for row in matrix:
                parent_item = row.get(parent_prefix)
                if parent_item and str(parent_item.uid) in link_map:
                    children = link_map[str(parent_item.uid)]
                    for i, child in enumerate(children):
                        if i == 0:
                            row[child_doc.prefix] = child
                        else:
                            new_row = dict(row)
                            new_row[child_doc.prefix] = child
                            new_rows.append(new_row)
            matrix.extend(new_rows)
            expand_children(child_doc, child_doc.prefix)

    for root_doc in root_docs:
        expand_children(root_doc, root_doc.prefix)

    return matrix, prefixes


def compute_coverage(tree):
    """ドキュメントペア別・グループ別のカバレッジを計算する。"""
    docs = {doc.prefix: doc for doc in tree}
    coverage = {}

    for doc in tree:
        if not doc.parent or doc.parent not in docs:
            continue
        parent_doc = docs[doc.parent]
        parent_uids = {str(item.uid) for item in parent_doc}
        covered_uids = set()

        for item in doc:
            for link in item.links:
                link_str = str(link)
                if link_str in parent_uids:
                    covered_uids.add(link_str)

        total = len(parent_uids)
        covered = len(covered_uids)
        pct = (covered / total * 100) if total > 0 else 0.0

        # グループ別
        group_cov = defaultdict(lambda: {"total": set(), "covered": set()})
        for pi in parent_doc:
            group_cov[get_group(pi)]["total"].add(str(pi.uid))
        for item in doc:
            for link in item.links:
                link_str = str(link)
                if link_str in parent_uids:
                    po = parent_doc.find_item(link_str)
                    group_cov[get_group(po)]["covered"].add(link_str)

        groups = {}
        for g, d in sorted(group_cov.items()):
            gt, gc = len(d["total"]), len(d["covered"])
            groups[g] = {
                "total": gt, "covered": gc, "uncovered": gt - gc,
                "percentage": round(gc / gt * 100, 1) if gt > 0 else 0.0,
                "uncovered_items": sorted(d["total"] - d["covered"]),
            }

        coverage[f"{doc.prefix} → {doc.parent}"] = {
            "total": total, "covered": covered, "uncovered": total - covered,
            "percentage": round(pct, 1),
            "uncovered_items": sorted(parent_uids - covered_uids),
            "by_group": groups,
        }

    return coverage


def _color(pct):
    if pct == 100: return "#4caf50"
    elif pct >= 50: return "#ff9800"
    else: return "#f44336"


def generate_html_report(tree, issues, matrix, prefixes, coverage, output_path):
    """グループフィルタ+レビュー状態つきHTMLレポートを生成する。"""
    now = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
    h = html.escape

    all_groups = sorted({get_group(item) for doc in tree for item in doc})

    group_buttons = ''.join(
        f'<button class="group-btn" data-group="{h(g)}" onclick="toggleGroup(this)">{h(g)}</button>'
        for g in all_groups
    )

    # レビュー統計
    total_items = sum(len(list(d)) for d in tree)
    reviewed_items = sum(
        1 for d in tree for item in d
        if item.reviewed is not None and item.reviewed != ""
    )

    # カバレッジテーブル
    coverage_rows = ""
    for pair, data in coverage.items():
        color = _color(data["percentage"])
        uncovered_str = ", ".join(data["uncovered_items"]) if data["uncovered_items"] else "—"
        coverage_rows += f"""
        <tr class="coverage-total">
            <td><strong>{h(pair)}</strong></td>
            <td>—</td>
            <td>{data['covered']} / {data['total']}</td>
            <td style="color:{color}; font-weight:bold">{data['percentage']}%</td>
            <td style="font-size:0.85em">{h(uncovered_str)}</td>
        </tr>"""
        for g, gd in data.get("by_group", {}).items():
            gc = _color(gd["percentage"])
            gu = ", ".join(gd["uncovered_items"]) if gd["uncovered_items"] else "—"
            coverage_rows += f"""
        <tr class="coverage-group" data-group="{h(g)}">
            <td style="padding-left:30px">{h(pair)}</td>
            <td><span class="group-tag">{h(g)}</span></td>
            <td>{gd['covered']} / {gd['total']}</td>
            <td style="color:{gc}; font-weight:bold">{gd['percentage']}%</td>
            <td style="font-size:0.85em">{h(gu)}</td>
        </tr>"""

    # イシュー一覧
    error_items = "".join(f"<li class='error'>{h(e)}</li>" for e in issues["errors"])
    warning_items = "".join(f"<li class='warning'>{h(w)}</li>" for w in issues["warnings"])
    info_items = "".join(f"<li class='info'>{h(i)}</li>" for i in issues["info"])
    issue_section = ""
    if issues["errors"]:
        issue_section += f"<h3>エラー ({len(issues['errors'])}件)</h3><ul>{error_items}</ul>"
    if issues["warnings"]:
        issue_section += f"<h3>警告 ({len(issues['warnings'])}件)</h3><ul>{warning_items}</ul>"
    if issues["info"]:
        issue_section += f"<h3>情報</h3><ul>{info_items}</ul>"
    if not issues["errors"] and not issues["warnings"]:
        issue_section = "<p style='color:#4caf50; font-weight:bold'>問題は検出されませんでした。</p>"
        if issues["info"]:
            issue_section += f"<h3>情報</h3><ul>{info_items}</ul>"

    # トレーサビリティマトリクス
    header_cells = "<th>グループ</th>" + "".join(f"<th>{h(p)}</th>" for p in prefixes)
    matrix_rows = ""
    for row in matrix:
        group = h(row.get("_group", "(未分類)"))
        cells = f'<td><span class="group-tag">{group}</span></td>'
        for prefix in prefixes:
            item = row.get(prefix)
            if item:
                text_preview = item.text[:80] + ("..." if len(item.text) > 80 else "")
                ref = get_ref(item)
                ref_html = f'<br><span class="ref-tag">{h(ref)}</span>' if ref else ""
                # レビュー状態
                rev = item.reviewed
                rev_icon = "✓" if (rev is not None and rev != "") else "○"
                rev_cls = "reviewed" if (rev is not None and rev != "") else "unreviewed"
                cells += (
                    f'<td><strong>{h(str(item.uid))}</strong> '
                    f'<span class="{rev_cls}">{rev_icon}</span>'
                    f'<br><span class="text-preview">{h(text_preview)}</span>'
                    f'{ref_html}</td>'
                )
            else:
                cells += '<td class="empty">—</td>'
        matrix_rows += f'<tr data-group="{group}">{cells}</tr>'

    report_html = f"""<!DOCTYPE html>
<html lang="ja">
<head>
<meta charset="UTF-8">
<title>トレーサビリティレポート</title>
<style>
  body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
         max-width: 1400px; margin: 0 auto; padding: 20px; background: #fafafa; }}
  h1 {{ border-bottom: 3px solid #1a73e8; padding-bottom: 10px; }}
  h2 {{ color: #1a73e8; margin-top: 30px; }}
  table {{ border-collapse: collapse; width: 100%; margin: 15px 0; background: #fff; }}
  th {{ background: #1a73e8; color: #fff; padding: 10px; text-align: left; }}
  td {{ border: 1px solid #ddd; padding: 8px; vertical-align: top; }}
  tr:nth-child(even) {{ background: #f8f9fa; }}
  .text-preview {{ color: #666; font-size: 0.85em; }}
  .ref-tag {{ display: inline-block; background: #f3e5f5; color: #7b1fa2; padding: 1px 6px;
              border-radius: 3px; font-size: 0.75em; font-family: monospace; margin-top: 2px; }}
  .empty {{ color: #ccc; text-align: center; }}
  .error {{ color: #d32f2f; }}
  .warning {{ color: #f57c00; }}
  .info {{ color: #1976d2; }}
  .reviewed {{ color: #4caf50; font-size: 0.8em; }}
  .unreviewed {{ color: #bdbdbd; font-size: 0.8em; }}
  .summary {{ display: flex; gap: 15px; margin: 15px 0; flex-wrap: wrap; }}
  .card {{ background: #fff; border: 1px solid #ddd; border-radius: 8px;
           padding: 15px 20px; flex: 1; min-width: 100px; text-align: center; }}
  .card h3 {{ margin: 0 0 5px; font-size: 0.85em; color: #666; }}
  .card .value {{ font-size: 1.6em; font-weight: bold; color: #1a73e8; }}
  .timestamp {{ color: #999; font-size: 0.85em; }}
  .group-tag {{ display: inline-block; background: #e3f2fd; color: #1565c0; padding: 2px 8px;
                border-radius: 4px; font-size: 0.8em; font-weight: bold; }}
  .group-filter {{ margin: 15px 0; }}
  .group-btn {{ margin: 3px; padding: 6px 14px; border: 1px solid #1a73e8; border-radius: 16px;
                background: #fff; color: #1a73e8; cursor: pointer; font-size: 0.85em; }}
  .group-btn.active {{ background: #1a73e8; color: #fff; }}
  .group-btn:hover {{ background: #e3f2fd; }}
  .group-btn.active:hover {{ background: #1565c0; }}
  .coverage-group {{ font-size: 0.9em; }}
  tr.hidden {{ display: none; }}
</style>
</head>
<body>
<h1>トレーサビリティレポート</h1>
<p class="timestamp">生成日時: {now}</p>

<div class="summary">
  <div class="card">
    <h3>ドキュメント</h3>
    <div class="value">{len(list(tree))}</div>
  </div>
  <div class="card">
    <h3>総アイテム</h3>
    <div class="value">{total_items}</div>
  </div>
  <div class="card">
    <h3>グループ</h3>
    <div class="value">{len(all_groups)}</div>
  </div>
  <div class="card">
    <h3>レビュー済</h3>
    <div class="value">{reviewed_items}/{total_items}</div>
  </div>
  <div class="card">
    <h3>エラー</h3>
    <div class="value" style="color:{'#d32f2f' if issues['errors'] else '#4caf50'}">{len(issues['errors'])}</div>
  </div>
  <div class="card">
    <h3>警告</h3>
    <div class="value" style="color:{'#f57c00' if issues['warnings'] else '#4caf50'}">{len(issues['warnings'])}</div>
  </div>
</div>

<h2>機能グループフィルタ</h2>
<div class="group-filter">
  <button class="group-btn active" data-group="__all__" onclick="showAll()">すべて</button>
  {group_buttons}
</div>

<h2>カバレッジ</h2>
<table id="coverage-table">
<tr><th>リンク方向</th><th>グループ</th><th>カバー数</th><th>カバー率</th><th>未カバーアイテム</th></tr>
{coverage_rows}
</table>

<h2>検証結果</h2>
{issue_section}

<h2>トレーサビリティマトリクス</h2>
<p style="font-size:0.85em; color:#888">✓=レビュー済　○=未レビュー</p>
<table id="matrix-table">
<tr>{header_cells}</tr>
{matrix_rows}
</table>

<script>
let activeGroups = new Set();
function toggleGroup(btn) {{
  const group = btn.dataset.group;
  const allBtn = document.querySelector('[data-group="__all__"]');
  if (activeGroups.has(group)) {{ activeGroups.delete(group); btn.classList.remove('active'); }}
  else {{ activeGroups.add(group); btn.classList.add('active'); allBtn.classList.remove('active'); }}
  if (activeGroups.size === 0) {{ showAll(); return; }}
  filterRows();
}}
function showAll() {{
  activeGroups.clear();
  document.querySelectorAll('.group-btn').forEach(b => b.classList.remove('active'));
  document.querySelector('[data-group="__all__"]').classList.add('active');
  document.querySelectorAll('#matrix-table tr[data-group]').forEach(r => r.classList.remove('hidden'));
  document.querySelectorAll('#coverage-table .coverage-group').forEach(r => r.classList.remove('hidden'));
  document.querySelectorAll('#coverage-table .coverage-total').forEach(r => r.classList.remove('hidden'));
}}
function filterRows() {{
  document.querySelectorAll('#matrix-table tr[data-group]').forEach(row => {{
    row.classList.toggle('hidden', !activeGroups.has(row.dataset.group));
  }});
  document.querySelectorAll('#coverage-table .coverage-group').forEach(row => {{
    row.classList.toggle('hidden', !activeGroups.has(row.dataset.group));
  }});
}}
</script>
</body>
</html>"""

    with open(output_path, "w", encoding="utf-8") as f:
        f.write(report_html)
    return output_path


def main():
    parser = argparse.ArgumentParser(
        description="Doorstopプロジェクトのバリデーションとレポート生成"
    )
    parser.add_argument("project_dir", help="プロジェクトのルートディレクトリ")
    parser.add_argument("--output-dir", default="./reports", help="レポート出力先")
    parser.add_argument("--strict", action="store_true",
                        help="全ての親アイテムに子リンクがあることを要求する")
    parser.add_argument("--json", action="store_true", help="JSON形式でもサマリを出力する")
    args = parser.parse_args()

    project_dir = os.path.abspath(args.project_dir)
    os.chdir(project_dir)

    print("ドキュメントツリーを構築中...")
    tree = doorstop.build()

    print("バリデーション実行中...")
    issues = validate_tree(tree, strict=args.strict, project_dir=project_dir)

    print("トレーサビリティマトリクスを構築中...")
    matrix, prefixes = build_traceability_matrix(tree)

    print("カバレッジを計算中...")
    coverage = compute_coverage(tree)

    output_dir = os.path.join(project_dir, args.output_dir)
    os.makedirs(output_dir, exist_ok=True)

    html_path = os.path.join(output_dir, "traceability_report.html")
    generate_html_report(tree, issues, matrix, prefixes, coverage, html_path)
    print(f"HTMLレポート: {html_path}")

    if args.json:
        total_items = sum(len(list(d)) for d in tree)
        reviewed_items = sum(
            1 for d in tree for item in d
            if item.reviewed is not None and item.reviewed != ""
        )
        summary = {
            "timestamp": datetime.now().isoformat(),
            "documents": {doc.prefix: len(list(doc)) for doc in tree},
            "groups": sorted({get_group(item) for doc in tree for item in doc}),
            "review_status": {"total": total_items, "reviewed": reviewed_items},
            "issues": issues,
            "coverage": coverage,
        }
        json_path = os.path.join(output_dir, "validation_summary.json")
        with open(json_path, "w", encoding="utf-8") as f:
            json.dump(summary, f, ensure_ascii=False, indent=2, default=list)
        print(f"JSONサマリ: {json_path}")

    # コンソールサマリ
    total_items = sum(len(list(d)) for d in tree)
    reviewed_items = sum(
        1 for d in tree for item in d
        if item.reviewed is not None and item.reviewed != ""
    )
    groups = sorted({get_group(item) for doc in tree for item in doc})

    print(f"\n===== バリデーション結果 =====")
    print(f"ドキュメント: {', '.join(d.prefix for d in tree)}")
    print(f"総アイテム数: {total_items}")
    print(f"機能グループ: {', '.join(groups)}")
    print(f"レビュー済: {reviewed_items}/{total_items}")
    print(f"エラー: {len(issues['errors'])}件  警告: {len(issues['warnings'])}件")

    if issues["errors"]:
        print("\n[エラー]")
        for e in issues["errors"]:
            print(f"  ✗ {e}")
    if issues["warnings"]:
        print("\n[警告]")
        for w in issues["warnings"]:
            print(f"  ⚠ {w}")
    if issues["info"]:
        print("\n[情報]")
        for i in issues["info"]:
            print(f"  ℹ {i}")

    print("\n[カバレッジ]")
    for pair, data in coverage.items():
        mark = "✓" if data["percentage"] == 100 else "△" if data["percentage"] >= 50 else "✗"
        print(f"  {mark} {pair}: {data['covered']}/{data['total']} ({data['percentage']}%)")
        if data.get("by_group"):
            for g, gd in data["by_group"].items():
                gm = "✓" if gd["percentage"] == 100 else "△" if gd["percentage"] >= 50 else "✗"
                print(f"      {gm} [{g}] {gd['covered']}/{gd['total']} ({gd['percentage']}%)")

    sys.exit(1 if issues["errors"] else 0)


if __name__ == "__main__":
    main()
