# rodf fidelity scoreboard

Generated 2026-08-23 09:12 UTC · oracle v2: 양쪽 PDF를 동일 래스터라이저(pdftoppm)로 비교 · pass = blur2 SSIM ≥ 0.95

`png blur2`는 참고 열: LO PNG 내보내기 vs rodf 자체 래스터 (힌팅/감마 차이 포함).

**6/10 PASS**

| doc | status | raw SSIM | blur2 SSIM | png blur2 | losses |
|---|---|---|---|---|---|
| bold-italic | FAIL | 0.7473 | 0.9245 | None | 0 |
| font-batang | PASS | 0.9736 | 0.9923 | None | 0 |
| font-gulim | PASS | 0.8547 | 0.9573 | None | 0 |
| mixed-script-sizes | PASS | 0.9446 | 0.9824 | None | 0 |
| multi-paragraph | FAIL | 0.7397 | 0.7965 | None | 0 |
| plain-en-12 | PASS | 0.9709 | 0.9885 | None | 0 |
| plain-ko-10 | PASS | 0.9129 | 0.9886 | None | 0 |
| size-ladder | PASS | 0.9836 | 0.9956 | None | 0 |
| wrap-korean | FAIL | 0.7872 | 0.8592 | None | 0 |
| wrap-mixed | FAIL | 0.6051 | 0.6948 | None | 0 |
