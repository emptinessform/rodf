# rodf fidelity scoreboard

Generated 2026-08-23 12:39 UTC · oracle v2: 양쪽 PDF를 동일 래스터라이저(pdftoppm)로 비교 · pass = blur2 SSIM ≥ 0.95

`png blur2`는 참고 열: LO PNG 내보내기 vs rodf 자체 래스터 (힌팅/감마 차이 포함).

**33/50 PASS** · 16 unsupported (coverage gaps)

| doc | status | raw SSIM | blur2 SSIM | png blur2 | coverage gaps |
|---|---|---|---|---|---|
| annotation-formatting | PASS | 0.9976 | 0.9975 | 0.9976 | forms x1, list x1 |
| BibliographyEntryField | UNSUPPORTED | 0.7616 | 0.7552 | 0.7513 | bibliography x1 |
| borders_ooo33 | UNSUPPORTED | 0.9222 | 0.9093 | 0.9095 | table x1 |
| dateFormFormats | UNSUPPORTED | 0.9537 | 0.9388 | 0.9608 | control x12, forms x1 |
| empty-svg-family-name | UNSUPPORTED | 0.9247 | 0.9232 | 0.923 | list x1 |
| fdo37606 | UNSUPPORTED | 0.7883 | 0.7632 | 0.7575 | forms x1, table x1 |
| fdo53210 | UNSUPPORTED | 0.9195 | 0.89 | 0.8955 | forms x1 |
| fdo55814 | UNSUPPORTED | 0.6771 | 0.6554 | 0.6491 | table x1 |
| fdo56272 | PASS | 0.9965 | 0.9969 | 0.997 | list x1 |
| fdo60842 | PASS | 0.9902 | 0.9887 | 0.9889 | table x1 |
| fdo68839 | PASS | 0.9886 | 0.9873 | 0.9874 | frame x4 |
| fdo69862 | PASS | 0.9762 | 0.9737 | 0.973 | forms x1, note x1, table x1 |
| fdo69979 | UNSUPPORTED | 0.8286 | 0.7784 | 0.7997 | table x1 |
| fdo75872_aoo40 | PASS | 0.9987 | 0.9988 | 0.999 | — |
| fdo75872_ooo33 | PASS | 0.9973 | 0.9965 | 0.9966 | — |
| fdo79269 | PASS | 0.9996 | 0.9996 | 0.9996 | — |
| fdo79269_header | PASS | 0.9996 | 0.9995 | 0.9995 | — |
| fdo81223 | PASS | 0.9944 | 0.9914 | 0.9913 | frame x1 |
| fdo82165 | PASS | 0.9907 | 0.9861 | 0.986 | — |
| fdo90130-1 | PASS | 0.9883 | 0.9872 | 0.9869 | frame x1 |
| fdo90130-2 | PASS | 0.9923 | 0.9907 | 0.9909 | frame x1 |
| feature_image_jpg | UNSUPPORTED | 0.9445 | 0.9424 | 0.9426 | frame x1 |
| feature_table | PASS | 1.0 | 1.0 | 1.0 | table x1 |
| feature_table_merged-cells | PASS | 1.0 | 1.0 | 1.0 | table x1 |
| feature_table_merged-cells_all | PASS | 1.0 | 1.0 | 1.0 | table x1 |
| feature_table_text | PASS | 0.9989 | 0.9986 | 0.9987 | table x1 |
| feature_text | PASS | 0.9851 | 0.9945 | 0.9776 | — |
| feature_text_background-color | PASS | 0.9738 | 0.9543 | 0.943 | — |
| feature_text_bold | PASS | 0.9792 | 0.9894 | 0.9661 | — |
| feature_text_italic | PASS | 0.9808 | 0.9949 | 0.9702 | — |
| hello | PASS | 0.9729 | 0.9927 | 0.9689 | — |
| incorrectsum | PASS | 0.9545 | 0.9522 | 0.9535 | table x1 |
| ooo32780-1 | UNSUPPORTED | 0.4964 | 0.5012 | 0.5029 | note x2 |
| ooo77837-1 | UNSUPPORTED | 0.5878 | 0.561 | 0.5696 | forms x1, list x1 |
| PageBackground | UNSUPPORTED | 0.3893 | 0.3927 | 0.4104 | frame x4 |
| paste-first-para-direct-format | PASS | 0.9736 | 0.996 | 0.9871 | — |
| space | FAIL | 0.6725 | 0.6795 | 0.6852 | — |
| spellmenu-redline | PASS | 0.911 | 0.9746 | 0.965 | — |
| tdf100033_1 | PASS | 0.9955 | 0.9938 | 0.9945 | frame x3 |
| tdf100033_2 | UNSUPPORTED | 0.8867 | 0.8558 | 0.859 | frame x3 |
| tdf101729 | PASS | 0.9722 | 0.9661 | 0.9667 | forms x1, table x1 |
| tdf103025 | PASS | 0.9814 | 0.9792 | 0.9796 | table x8 |
| tdf107392 | UNSUPPORTED | 0.9435 | 0.9426 | 0.9442 | frame x3 |
| tdf108482 | PASS | 0.9828 | 0.9762 | 0.9775 | forms x1, table x1 |
| tdf109080_loext_ns | PASS | 0.9603 | 0.9616 | 0.9615 | — |
| tdf109080_style_ns | PASS | 0.9603 | 0.9616 | 0.9615 | — |
| tdf109228 | PASS | 0.9915 | 0.9903 | 0.9903 | frame x1 |
| tdf113289 | PASS | 0.9907 | 0.9862 | 0.9859 | note x1 |
| Word2010AsCharShape | UNSUPPORTED | 0.605 | 0.6165 | 0.6233 | custom-shape x1 |
| ZoneMacroTest | UNSUPPORTED | 0.7926 | 0.7751 | 0.7781 | forms x1, table x1 |
