#!/usr/bin/env python
"""rodf 코퍼스 스코어보드 — corpus/*.odt 전체를 오라클로 비교해 집계한다.

설계 문서 원칙: 코퍼스 CI는 게이트가 아니라 **스코어보드**. 미지원 요소 문서는
fail이 아닌 커버리지 미달로 기록하고, "동작"의 기준은 크래시 0 + 결정론이다.

사용법: python tools/corpus.py [--threshold 0.95] [--out docs/scoreboard.md]
"""

import argparse
import json
import subprocess
import tempfile
from datetime import datetime, timezone
from pathlib import Path

import numpy as np
from PIL import Image, ImageFilter

from oracle import content_bbox, register, render_libreoffice, ssim

ROOT = Path(__file__).resolve().parent.parent
CORPUS = ROOT / "corpus"


def render_rodf(odt: Path, out_png: Path) -> subprocess.CompletedProcess:
    return subprocess.run(
        ["cargo", "run", "-q", "-p", "rodf-cli", "--", "render", str(odt), str(out_png)],
        cwd=ROOT, capture_output=True, text=True,
    )


def compare(odt: Path) -> dict:
    result = {"doc": odt.stem}
    with tempfile.TemporaryDirectory() as tmp:
        tmp_dir = Path(tmp)
        rodf_png = tmp_dir / "rodf.png"
        proc = render_rodf(odt, rodf_png)
        if proc.returncode != 0 or not rodf_png.exists():
            result["status"] = "CRASH"
            result["detail"] = (proc.stderr or "").strip()[-200:]
            return result
        losses = [l for l in (proc.stderr or "").splitlines() if "mapping loss" in l]
        result["losses"] = len(losses)

        rodf_img = Image.open(rodf_png).convert("L")
        try:
            lo_png = render_libreoffice(odt, tmp_dir, *rodf_img.size)
            lo_img = Image.open(lo_png).convert("L")
        except Exception as e:
            result["status"] = "ORACLE-ERROR"
            result["detail"] = str(e)[:200]
            return result

        if lo_img.size != rodf_img.size:
            w = min(lo_img.width, rodf_img.width)
            h = min(lo_img.height, rodf_img.height)
            lo_img = lo_img.crop((0, 0, w, h))
            rodf_img = rodf_img.crop((0, 0, w, h))

        a, b = np.asarray(lo_img), np.asarray(rodf_img)
        _, _, b = register(a, b)
        ax = content_bbox(a)
        bx = content_bbox(b)
        x0, y0 = min(ax[0], bx[0]), min(ax[1], bx[1])
        x1, y1 = max(ax[2], bx[2]), max(ax[3], bx[3])
        for radius, key in ((0, "raw"), (2, "blur2")):
            ai = Image.fromarray(a).filter(ImageFilter.GaussianBlur(radius)) if radius else Image.fromarray(a)
            bi = Image.fromarray(b).filter(ImageFilter.GaussianBlur(radius)) if radius else Image.fromarray(b)
            result[key] = round(
                ssim(np.asarray(ai)[y0:y1, x0:x1], np.asarray(bi)[y0:y1, x0:x1]), 4
            )
    return result


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--threshold", type=float, default=0.95)
    parser.add_argument("--out", type=Path, default=ROOT / "docs" / "scoreboard.md")
    parser.add_argument("--json", type=Path, default=ROOT / "docs" / "scoreboard.json")
    args = parser.parse_args()

    docs = sorted(CORPUS.glob("*.odt"))
    if not docs:
        raise SystemExit("corpus/ 에 .odt가 없습니다 — tools/gen_corpus.py 먼저 실행")

    rows = []
    for odt in docs:
        r = compare(odt)
        if "status" not in r:
            r["status"] = "PASS" if r["blur2"] >= args.threshold else "FAIL"
        rows.append(r)
        print(f'{r["doc"]:<22}{r["status"]:<14}raw={r.get("raw","-"):<9}blur2={r.get("blur2","-")}')

    passed = sum(1 for r in rows if r["status"] == "PASS")
    stamp = datetime.now(timezone.utc).strftime("%Y-%m-%d %H:%M UTC")

    lines = [
        "# rodf fidelity scoreboard",
        "",
        f"Generated {stamp} · oracle: local LibreOffice · pass = blur2 SSIM ≥ {args.threshold}",
        "",
        f"**{passed}/{len(rows)} PASS**",
        "",
        "| doc | status | raw SSIM | blur2 SSIM | losses |",
        "|---|---|---|---|---|",
    ]
    for r in rows:
        lines.append(
            f'| {r["doc"]} | {r["status"]} | {r.get("raw", "—")} | '
            f'{r.get("blur2", "—")} | {r.get("losses", "—")} |'
        )
    lines.append("")
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text("\n".join(lines), encoding="utf-8")
    args.json.write_text(
        json.dumps({"generated": stamp, "threshold": args.threshold, "results": rows},
                   ensure_ascii=False, indent=2),
        encoding="utf-8",
    )
    print(f"\n{passed}/{len(rows)} PASS -> {args.out}")


if __name__ == "__main__":
    main()
