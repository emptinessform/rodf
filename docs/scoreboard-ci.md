# rodf fidelity scoreboard

Generated 2026-08-23 09:37 UTC · oracle v2: 양쪽 PDF를 동일 래스터라이저(pdftoppm)로 비교 · pass = blur2 SSIM ≥ 0.95

`png blur2`는 참고 열: LO PNG 내보내기 vs rodf 자체 래스터 (힌팅/감마 차이 포함).

**9/10 PASS**

| doc | status | raw SSIM | blur2 SSIM | png blur2 | losses |
|---|---|---|---|---|---|
| bold-italic | PASS | 0.9708 | 0.9902 | 0.9521 | 0 |
| font-batang | PASS | 0.9913 | 0.9962 | 0.9798 | 0 |
| font-gulim | PASS | 0.9021 | 0.9765 | 0.9588 | 0 |
| mixed-script-sizes | PASS | 0.9546 | 0.9841 | 0.9829 | 0 |
| multi-paragraph | FAIL | 0.8857 | 0.9201 | 0.9106 | 0 |
| plain-en-12 | PASS | 0.9787 | 0.9927 | 0.9754 | 0 |
| plain-ko-10 | PASS | 0.9129 | 0.9886 | 0.9726 | 0 |
| size-ladder | PASS | 0.9884 | 0.9974 | 0.9867 | 0 |
| wrap-korean | PASS | 0.8858 | 0.9799 | 0.9843 | 0 |
| wrap-mixed | PASS | 0.9088 | 0.9648 | 0.9807 | 0 |
