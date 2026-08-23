# rodf fidelity scoreboard

Generated 2026-08-23 08:31 UTC · oracle v2: 양쪽 PDF를 동일 래스터라이저(pdftoppm)로 비교 · pass = blur2 SSIM ≥ 0.95

`png blur2`는 참고 열: LO PNG 내보내기 vs rodf 자체 래스터 (힌팅/감마 차이 포함).

**10/10 PASS**

| doc | status | raw SSIM | blur2 SSIM | png blur2 | losses |
|---|---|---|---|---|---|
| bold-italic | PASS | 0.9898 | 0.9979 | 0.9737 | 0 |
| font-batang | PASS | 0.9005 | 0.9851 | 0.9482 | 0 |
| font-gulim | PASS | 0.8648 | 0.9759 | 0.9555 | 0 |
| mixed-script-sizes | PASS | 0.9464 | 0.9857 | 0.9897 | 0 |
| multi-paragraph | PASS | 0.9456 | 0.9898 | 0.9726 | 0 |
| plain-en-12 | PASS | 0.9829 | 0.9957 | 0.9694 | 0 |
| plain-ko-10 | PASS | 0.9446 | 0.992 | 0.9626 | 0 |
| size-ladder | PASS | 0.9436 | 0.989 | 0.9875 | 0 |
| wrap-korean | PASS | 0.9915 | 0.9984 | 0.9451 | 0 |
| wrap-mixed | PASS | 0.9903 | 0.9984 | 0.9547 | 0 |
