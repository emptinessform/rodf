#!/usr/bin/env python
"""rodf 오라클 코퍼스 생성기 — 지원 기능 매트릭스를 커버하는 ODT 픽스처를
LibreOffice로 저작해 corpus/에 저장한다.

사용법: python tools/gen_corpus.py
"""

import subprocess
import sys
from pathlib import Path

SOFFICE = r"C:\Program Files\LibreOffice\program\soffice.exe"
ROOT = Path(__file__).resolve().parent.parent
CORPUS = ROOT / "corpus"

HEAD = """<?xml version="1.0" encoding="UTF-8"?>
<office:document xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0"
 xmlns:style="urn:oasis:names:tc:opendocument:xmlns:style:1.0"
 xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0"
 xmlns:fo="urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0"
 xmlns:svg="urn:oasis:names:tc:opendocument:xmlns:svg-compatible:1.0"
 office:version="1.3" office:mimetype="application/vnd.oasis.opendocument.text">
  <office:font-face-decls>
    <style:font-face style:name="맑은 고딕" svg:font-family="'맑은 고딕'"/>
    <style:font-face style:name="굴림" svg:font-family="'굴림'"/>
    <style:font-face style:name="바탕" svg:font-family="'바탕'"/>
  </office:font-face-decls>
  <office:automatic-styles>
{styles}
  </office:automatic-styles>
  <office:body><office:text>
{paras}
  </office:text></office:body>
</office:document>"""


def pstyle(name, font="맑은 고딕", size=None, size_asian=None, bold=False, italic=False):
    attrs = [f'style:font-name="{font}"', f'style:font-name-asian="{font}"']
    if size is not None:
        attrs.append(f'fo:font-size="{size}pt"')
    if size_asian is not None:
        attrs.append(f'style:font-size-asian="{size_asian}pt"')
    if bold:
        attrs.append('fo:font-weight="bold" style:font-weight-asian="bold"')
    if italic:
        attrs.append('fo:font-style="italic" style:font-style-asian="italic"')
    return (
        f'    <style:style style:name="{name}" style:family="paragraph">\n'
        f'      <style:text-properties {" ".join(attrs)}/>\n'
        f"    </style:style>"
    )


def p(text, style=None):
    s = f' text:style-name="{style}"' if style else ""
    return f"    <text:p{s}>{text}</text:p>"


LONG_KO = (
    "문서 포맷의 렌더링 충실도는 결국 디테일의 총합이다. 줄바꿈 위치 하나, "
    "자간 한 픽셀, 행간의 미세한 차이가 쌓여 전체 인상을 결정한다. "
    "rodf는 이 디테일을 LibreOffice 오라클과의 픽셀 비교로 검증한다."
)
LONG_MIX = (
    "OpenDocument Format은 ISO/IEC 26300 국제 표준이며, content.xml과 "
    "styles.xml의 automatic style 체인을 해석해야 한다. Korean typography는 "
    "first-class requirement이고, 줄바꿈은 한글과 Latin이 섞일 때 가장 어렵다."
)

DOCS = {
    # 기본형: 단일 크기·단일 문단
    "plain-ko-10": ([pstyle("P1", size=10, size_asian=10)], [p("한글 본문 한 문단입니다.", "P1")]),
    "plain-en-12": ([pstyle("P1", size=12, size_asian=12)], [p("A single English paragraph.", "P1")]),
    # 스타일 변형
    "bold-italic": (
        [pstyle("P1", size=14, size_asian=14, bold=True), pstyle("P2", size=14, size_asian=14, italic=True)],
        [p("굵은 강조 Bold 문단", "P1"), p("기울임 Italic 문단입니다", "P2")],
    ),
    "size-ladder": (
        [pstyle(f"S{s}", size=s, size_asian=s) for s in (8, 10, 12, 16, 24, 36)],
        [p(f"{s}pt 크기 Size sample 한글", f"S{s}") for s in (8, 10, 12, 16, 24, 36)],
    ),
    # 문자체계 분리 (서양/asian 크기 상이)
    "mixed-script-sizes": (
        [pstyle("P1", size=24, size_asian=10, bold=True)],
        [p("안녕하세요 Hello 혼합 Mixed", "P1")],
    ),
    # 폰트별 (lineGap 지형: 맑은고딕 gap0 / 굴림·바탕 gap152)
    "font-gulim": ([pstyle("P1", font="굴림", size=14, size_asian=14)], [p("굴림 글꼴 문단 Gulim paragraph", "P1")] * 3),
    "font-batang": ([pstyle("P1", font="바탕", size=14, size_asian=14)], [p("바탕 글꼴 문단 Batang paragraph", "P1")] * 3),
    # 줄바꿈 (여러 줄로 감김)
    "wrap-korean": ([pstyle("P1", size=12, size_asian=12)], [p(LONG_KO, "P1")]),
    "wrap-mixed": ([pstyle("P1", size=12, size_asian=12)], [p(LONG_MIX, "P1")]),
    # 다문단 문서
    "multi-paragraph": (
        [pstyle("H", size=18, size_asian=18, bold=True), pstyle("B", size=11, size_asian=11)],
        [p("제목 문단 Heading", "H"), p(LONG_KO, "B"), p(LONG_MIX, "B"), p("마지막 문단 The end.", "B")],
    ),
}


def main() -> None:
    CORPUS.mkdir(exist_ok=True)
    for name, (styles, paras) in DOCS.items():
        fodt = CORPUS / f"{name}.fodt"
        fodt.write_text(
            HEAD.format(styles="\n".join(styles), paras="\n".join(paras)),
            encoding="utf-8",
        )
        subprocess.run(
            [SOFFICE, "--headless", "--convert-to", "odt", str(fodt), "--outdir", str(CORPUS)],
            check=True, capture_output=True,
        )
        print(f"generated corpus/{name}.odt")
    print(f"{len(DOCS)} documents")


if __name__ == "__main__":
    main()
