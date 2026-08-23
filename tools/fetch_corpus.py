#!/usr/bin/env python
"""와일드 ODT 코퍼스 수집기 — LibreOffice core 저장소의 odfimport 테스트
문서(MPL-2.0)를 커밋 고정 URL로 내려받고 출처 대장을 생성한다.

사용법: python tools/fetch_corpus.py [--count 50] [--ref libreoffice-26.2.1.2]
"""

import argparse
import hashlib
import json
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
REPO = "LibreOffice/core"
DATA_DIR = "sw/qa/extras/odfimport/data"
MAX_BYTES = 100_000  # 소형 문서만 — 스코어보드 실행 시간 통제


def gh_json(url: str):
    req = urllib.request.Request(url, headers={"User-Agent": "rodf-corpus-fetch"})
    with urllib.request.urlopen(req) as r:
        return json.load(r)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--count", type=int, default=50)
    parser.add_argument("--ref", default="libreoffice-26.2.1.2")
    parser.add_argument("--out", type=Path, default=ROOT / "corpus-wild")
    args = parser.parse_args()

    listing = gh_json(
        f"https://api.github.com/repos/{REPO}/contents/{DATA_DIR}?ref={args.ref}"
    )
    picked = [
        f for f in listing
        if f["name"].endswith(".odt") and 0 < f["size"] <= MAX_BYTES
    ]
    picked.sort(key=lambda f: f["name"])
    picked = picked[: args.count]

    args.out.mkdir(parents=True, exist_ok=True)
    rows = []
    for f in picked:
        url = f"https://raw.githubusercontent.com/{REPO}/{args.ref}/{DATA_DIR}/{f['name']}"
        dest = args.out / f["name"]
        req = urllib.request.Request(url, headers={"User-Agent": "rodf-corpus-fetch"})
        with urllib.request.urlopen(req) as r:
            data = r.read()
        dest.write_bytes(data)
        sha = hashlib.sha256(data).hexdigest()
        rows.append((f["name"], len(data), sha, url))
        print(f"fetched {f['name']} ({len(data)}B)")

    prov = ROOT / "docs" / "CORPUS-PROVENANCE.md"
    lines = [
        "# 와일드 코퍼스 출처 대장",
        "",
        f"출처: [{REPO}](https://github.com/{REPO}) `{DATA_DIR}` @ `{args.ref}` (커밋 고정 태그)",
        "라이선스: **MPL-2.0** (LibreOffice 프로젝트 테스트 데이터 — 고지와 함께 재배포 가능)",
        f"수집 도구: tools/fetch_corpus.py · 파일 {len(rows)}개, 각 ≤ {MAX_BYTES // 1000}KB",
        "",
        "| file | bytes | sha256 |",
        "|---|---|---|",
    ]
    for name, size, sha, url in rows:
        lines.append(f"| [{name}]({url}) | {size} | `{sha[:16]}…` |")
    lines.append("")
    prov.write_text("\n".join(lines), encoding="utf-8")
    print(f"{len(rows)} files -> {args.out}, provenance -> {prov}")


if __name__ == "__main__":
    main()
