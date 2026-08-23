#!/usr/bin/env python
"""rodf 오라클 — LibreOffice 렌더와 rodf 렌더의 콘텐츠 크롭 SSIM 비교.

사용법:
    python tools/oracle.py <input.odt> [--dpi 144] [--threshold 0.95]

설계 문서 기준: 페이지 전체가 아닌 콘텐츠 바운딩 박스 크롭 후 SSIM
(희소 문서에서 여백이 점수를 지배하는 오탐 방지). M1은 로컬 soffice를
사용하고, M1.5에서 LO 버전 고정 Docker + 폰트 고정으로 결정론을 확보한다.
"""

import argparse
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

import numpy as np
from PIL import Image, ImageFilter

SOFFICE_CANDIDATES = [
    "soffice",
    r"C:\Program Files\LibreOffice\program\soffice.exe",
]


def find_soffice() -> str:
    for candidate in SOFFICE_CANDIDATES:
        if shutil.which(candidate) or Path(candidate).exists():
            return candidate
    sys.exit("oracle: LibreOffice(soffice)를 찾을 수 없습니다")


def render_libreoffice(odt: Path, out_dir: Path, width_px: int, height_px: int) -> Path:
    filter_opts = (
        f'png:writer_png_Export:{{"PixelWidth":{{"type":"long","value":{width_px}}},'
        f'"PixelHeight":{{"type":"long","value":{height_px}}}}}'
    )
    subprocess.run(
        [find_soffice(), "--headless", "--convert-to", filter_opts,
         str(odt), "--outdir", str(out_dir)],
        check=True, capture_output=True,
    )
    return out_dir / (odt.stem + ".png")


def render_rodf(odt: Path, out_png: Path) -> None:
    root = Path(__file__).resolve().parent.parent
    subprocess.run(
        ["cargo", "run", "-q", "-p", "rodf-cli", "--", "render", str(odt), str(out_png)],
        check=True, cwd=root,
    )


def content_bbox(gray: np.ndarray, threshold: int = 245) -> tuple[int, int, int, int]:
    """흰 배경이 아닌 픽셀의 바운딩 박스 (x0, y0, x1, y1)."""
    mask = gray < threshold
    if not mask.any():
        return (0, 0, gray.shape[1], gray.shape[0])
    ys, xs = np.where(mask)
    return (int(xs.min()), int(ys.min()), int(xs.max()) + 1, int(ys.max()) + 1)


def ssim(a: np.ndarray, b: np.ndarray) -> float:
    """단일 스케일 전역 가우시안 근사 없이 창 기반 SSIM (8x8 블록 평균)."""
    a = a.astype(np.float64)
    b = b.astype(np.float64)
    c1, c2 = (0.01 * 255) ** 2, (0.03 * 255) ** 2

    def blocks(x: np.ndarray) -> np.ndarray:
        h, w = x.shape
        h8, w8 = h - h % 8, w - w % 8
        return x[:h8, :w8].reshape(h8 // 8, 8, w8 // 8, 8)

    ba, bb = blocks(a), blocks(b)
    mu_a = ba.mean(axis=(1, 3))
    mu_b = bb.mean(axis=(1, 3))
    var_a = ba.var(axis=(1, 3))
    var_b = bb.var(axis=(1, 3))
    cov = (ba * bb).mean(axis=(1, 3)) - mu_a * mu_b
    s = ((2 * mu_a * mu_b + c1) * (2 * cov + c2)) / (
        (mu_a**2 + mu_b**2 + c1) * (var_a + var_b + c2)
    )
    return float(s.mean())


def register(a: np.ndarray, b: np.ndarray, radius: int = 2) -> tuple[int, int, np.ndarray]:
    """b를 ±radius px 이동시키며 a와의 오차를 최소화하는 전역 정합.

    여백 반올림 등에서 오는 1px 수준의 전역 오프셋은 충실도 결함이 아니므로
    비교 전에 흡수한다 (시각 회귀 테스트의 표준 관행)."""
    best = (0, 0, b)
    best_err = None
    af = a.astype(np.int32)
    for dy in range(-radius, radius + 1):
        for dx in range(-radius, radius + 1):
            shifted = np.roll(np.roll(b, dy, axis=0), dx, axis=1)
            err = np.abs(af - shifted.astype(np.int32)).mean()
            if best_err is None or err < best_err:
                best_err = err
                best = (dx, dy, shifted)
    return best


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("input", type=Path)
    parser.add_argument("--dpi", type=float, default=144.0)
    parser.add_argument("--threshold", type=float, default=0.95)
    parser.add_argument("--keep", type=Path, help="비교 이미지를 남길 디렉터리")
    args = parser.parse_args()

    with tempfile.TemporaryDirectory() as tmp:
        tmp_dir = Path(tmp)
        rodf_png = tmp_dir / "rodf.png"
        render_rodf(args.input, rodf_png)
        rodf_img = Image.open(rodf_png).convert("L")

        lo_png = render_libreoffice(args.input, tmp_dir, *rodf_img.size)
        lo_img = Image.open(lo_png).convert("L")
        if lo_img.size != rodf_img.size:
            # LO의 px 크기 반올림은 세션에 따라 ±1px 흔들린다. 리샘플은 전체를
            # 흐려 점수를 왜곡하므로, 초과분을 잘라내는 것으로 정규화한다.
            w = min(lo_img.width, rodf_img.width)
            h = min(lo_img.height, rodf_img.height)
            print(f"note: size mismatch LO{lo_img.size} vs rodf{rodf_img.size} -> crop to {w}x{h}")
            lo_img = lo_img.crop((0, 0, w, h))
            rodf_img = rodf_img.crop((0, 0, w, h))

        a = np.asarray(lo_img)
        b = np.asarray(rodf_img)

        dx, dy, b = register(a, b)
        print(f"registration: dx={dx} dy={dy}")

        # 두 렌더의 콘텐츠 바운딩 박스 합집합으로 크롭.
        ax0, ay0, ax1, ay1 = content_bbox(a)
        bx0, by0, bx1, by1 = content_bbox(b)
        x0, y0 = min(ax0, bx0), min(ay0, by0)
        x1, y1 = max(ax1, bx1), max(ay1, by1)
        print(f"content bbox: ({x0},{y0})-({x1},{y1})")

        # raw는 참고용, 판정은 blur2 기준 — 래스터라이저 AA 차이를 완화해
        # 레이아웃/글리프 배치 차이를 측정한다 (설계 문서 오라클 방법론).
        scores = {}
        for radius in (0, 1, 2):
            ai = Image.fromarray(a).filter(ImageFilter.GaussianBlur(radius)) if radius else Image.fromarray(a)
            bi = Image.fromarray(b).filter(ImageFilter.GaussianBlur(radius)) if radius else Image.fromarray(b)
            scores[radius] = ssim(
                np.asarray(ai)[y0:y1, x0:x1], np.asarray(bi)[y0:y1, x0:x1]
            )
        verdict = "PASS" if scores[2] >= args.threshold else "FAIL"
        print(
            f"content-cropped SSIM: raw {scores[0]:.4f} | blur1 {scores[1]:.4f} | "
            f"blur2 {scores[2]:.4f} (threshold {args.threshold} @blur2) -> {verdict}"
        )

        if args.keep:
            args.keep.mkdir(parents=True, exist_ok=True)
            Image.open(lo_png).save(args.keep / "oracle-libreoffice.png")
            Image.open(rodf_png).save(args.keep / "oracle-rodf.png")

    sys.exit(0 if verdict == "PASS" else 1)


if __name__ == "__main__":
    main()
