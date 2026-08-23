# rodf 진행 관리

> 실행 추적 문서 — 설계·결정의 "왜"는 [DESIGN.md](DESIGN.md), 충실도 현황은
> [scoreboard.md](scoreboard.md). **작업이 끝날 때마다 이 문서를 같은 커밋
> 흐름으로 갱신한다** (완료 항목은 날짜·커밋과 함께 완료 기록으로 내린다).

## 마일스톤 보드

| 마일스톤 | 상태 | 완료일 |
|---|---|---|
| M0 — 엔진 분리성 스파이크 | ✅ 완료 | 2026-08-23 |
| M1 — 파싱→렌더→오라클 SSIM ≥ 0.95 | ✅ 완료 | 2026-08-23 |
| M1.5 — 오라클 코퍼스 + CI | 🔶 진행 중 (1차 슬라이스 완료) | — |
| M2 — rlayout 승격 (D13) | ✅ 완료 | 2026-08-23 |
| M2.5 — 공개 충실도 대시보드 | ⬜ 대기 | — |
| M3+ — 표/이미지/헤더푸터, SVG, ODS/ODP, WASM | ⬜ 대기 | — |

## 지금 작업 (in progress)

(없음 — 다음 착수 시 대기열에서 승격)

## 대기열 (우선순위순)
- [ ] **rodf 자체 래스터의 감마/힌팅** (선택) — png 경로 0.947~0.99. 최종 사용자
  시각 동등성 항목. GDI식 감마 블렌딩 근사를 tiny-skia 경로에 적용할지 검토.
- [ ] **CI 렌더 테스트 안정화** — 러너 한국어 폰트 문제, OFL 폰트 번들 검토.

## 완료 기록 (최신순)

### 2026-08-23

- ✅ **와일드 FAIL 7건 전부 해소 — 33 PASS / 17 UNSUPPORTED / 0 FAIL / 0 CRASH**
  진단: 요소 히스토그램으로 7건 분류. 수정 3건: ① **문단 정렬**(fo:text-align
  파싱→rlayout Center/End/Justify — paste 문서 0.55의 정체) ② **빈 문단 보존**
  (<text:p/> Empty 폼이 통째로 사라져 행이 붕괴하던 결함) ③ **커버리지 확장**
  (note 스킵+집계로 각주가 본문에 섞이던 것 차단, bibliography/index류/
  custom-shape/object/control/forms/tab 보고, section은 투명 컨테이너).
  나머지 4건은 UNSUPPORTED로 정확 분류. 테스트 35개.
- ✅ **와일드 ODT 코퍼스 (M1.5 완결)** — LO core odfimport 테스트 ODT
  (MPL-2.0, libreoffice-26.2.1.2 태그 고정) 50개 수집기(fetch_corpus.py) +
  출처 대장(CORPUS-PROVENANCE.md, 파일별 sha256). 선행 구현: rodf-core
  **커버리지 감지**(미지원 table/frame/list/image 서브트리 스킵+집계,
  coverage_notes API) + **text:h 문단 처리**(조용히 버려지던 제목 복구) +
  템플릿 mimetype 허용 + 스코어보드 UNSUPPORTED 상태.
  **첫 실전 결과: 32 PASS / 11 UNSUPPORTED / 7 FAIL / 크래시 0.**
  FAIL 7건은 대기열 등재 (인라인 요소 계열).
- ✅ **행말 공백 오버플로 규칙 — 양 프로파일 10/10 달성** — multi-paragraph
  진단(밴드 7행 y 전부 일치, LONG_KO 첫 줄바꿈만 한 어절 차이)으로 적합
  판정이 어절의 꼬리 공백 폭까지 요구하는 결함 발견. LO·Word·CSS 공통 관행
  (행말 공백은 여백에 매달림, 잉크 없음)대로 적합 검사에서만 차감(포크
  4625463). 셰이핑 어드밴스 불변 유지(에디터 불변식), 각주 여백 가드는 잉크
  기준으로 수정. **noto 프로파일 multi-paragraph 0.920→0.995, 결정론
  스코어보드 10/10 · Windows 프로파일 10/10 유지.**
- ✅ **오라클 결정론 (M1.5 잔여)** — oracle/Dockerfile: rust 1.95 + **TDF
  아카이브 LO 26.2.1.2 버전 고정**(호스트와 동일 버전 — 프로파일 간 비교
  가능) + Noto CJK KR + poppler 22.12. 코퍼스를 컨테이너 안에서 noto 폰트
  셋으로 생성(gen_corpus --font-set) → 호스트 무관 재현. 주간 CI(oracle.yml)
  + 로컬 Docker 실검증. png 참고 열은 환경 결측 허용.
  **noto 프로파일 9/10 PASS.** 교훈: bookworm 기본 LO 7.4(2022)로 첫 실행 시
  6/10이었는데 구식 Type1 서브세터·구 한글 동작이 원인 — 배포판 LO가 아니라
  타깃 세대의 LO를 고정해야 한다.
  로컬 실행: docker run -v <repo>:/work -v rodf-oracle-target:/tmp/target rodf-oracle
- ✅ **M2: rlayout 승격 (D13)** — 중립 문서 IR + 문단 플로우 엔진
  `crates/rlayout` 신규(oxml-layout 직결, LO/ODF 관례 기본값). rodf-render를
  CT_* 어댑터에서 IR 매핑으로 전환, rdocx-layout/rdocx-oxml 의존 소멸,
  sentinel 불필요. 테스트 26개 + **스코어보드 10/10 유지**(픽셀 등가 검증).
  rpptx 사전 분할 경로도 미사용 — 어절 줄바꿈이 oxml-layout 필터만으로 동작.
- ✅ **AA/힌팅 격차 분석 → 오라클 v2** — 잉크 통계로 원인 확정(LO 글리프가
  일관되게 ~10% 더 진함 = GDI/DWrite 감마·힌팅). **오라클 v2: 양쪽 PDF를 동일
  래스터라이저(pdftoppm)로 비교** — 래스터라이저 변수를 상쇄해 순수 레이아웃
  충실도를 측정. 결과: **10/10 PASS** (wrap-korean raw 0.9915, batang blur2
  0.9851). png 경로는 참고 열로 유지. 충실도 백로그 전부 해소.
- ✅ **한글 어절 단위 줄바꿈** — LO는 어절(공백) 단위 + 무공백 음절열 폴백,
  엔진은 UAX14 음절 단위(Word 정답)였음을 판별 실험으로 확정. font-natural
  모드에 hangul_word_wrap 도입(포크 dbc756f) — text_segments 사전 분할과
  세그먼트 내 기회 스캔 양쪽에서 한글|한글 경계 억제. wrap-word 픽셀 테스트
  (우측 경계 987±12). 줄바꿈 위치 LO와 완전 일치. 스코어보드 wrap-korean
  0.84→0.947, multi-paragraph 0.90→**0.96 PASS**, 전체 **8/10**.
- ✅ **합성 기울임(synthetic oblique)** — 이탤릭 페이스 없는 폰트에서 20°
  (DirectWrite 관례, LO 실측 tan 0.358) 스큐 합성. 공유 계층 구현: FontData에
  synthetic_italic 플래그 + 래스터 글리프 변환·PDF Tm 스큐 (포크 1a7a5db로 정식 재수록 — 세션 충돌로 38f2aa8에 쓸렸다 235d56f에서 백아웃된 것을 재랜딩).
  스코어보드 bold-italic 0.69→**0.97 PASS**, 전체 6/10→**7/10**.
- ✅ 설계 문서 저장소 이관 (docs/DESIGN.md 정본화) — `af1cb9e`
- ✅ 용어 확정: FSL → **MCFS**(크레이트 rmcf), 문서 전면 반영 — `c1bea82` (D12)
- ✅ MCFS 라이선스 분석 문서 (docs/mcfs-licensing.md) — `3941085`
- ✅ M1.5 1차 슬라이스: 코퍼스 10문서 + 스코어보드(6/10 PASS) + GitHub Actions — `652b73f`
- ✅ 오라클 측정 버그 2건 수정(crop 정규화, ±2px 정합) → **M1 기준 달성**
  (hello blur2 0.9829, gulim 0.9764) — `a85ca49`
- ✅ 엔진 의존을 포크 git 의존(rev 고정)으로 전환 — 제3자 빌드 가능 — `1bbbf81`
- ✅ lineGap 결함 수정: 포크에 font-natural line rule(4ab9020) + rodf 적용,
  굴림 픽셀 테스트 2건 — `4ac19f1`
- ✅ 행간 모델 실험: LO 모델 = hhea asc+desc+lineGap 규명, 폰트명 미해석이
  진범임을 실증(β 증거 1호 철회), 픽스처 수정 — `cdad214`
- ✅ M1 파이프라인: rodf-core/render/cli + 오라클 + side-by-side, 테스트 TDD —
  `6e6b36b`~`4ac19f1`
- ✅ GitHub 공개(emptinessform/rodf), README 한/영 병기, 듀얼 라이선스 —
  `70dcd7c`~`a4493c3`
- ✅ M0 스파이크: 엄격 중립성 실패/어댑터 저비용 실증, oxml-layout 중립 계층 발견
- ✅ office-hours 설계 세션: 문제 정의 → 전제 검증 → 접근 확정 → 3라운드 리뷰 승인

## 운영 규칙

1. 작업 시작 시 "지금 작업"에 올리고, 완료 시 날짜·커밋 해시와 함께 완료
   기록으로 내린다.
2. 새 발견(스코어보드 실패, 실험 결과)은 대기열에 우선순위와 함께 추가한다.
3. 방향·구조 결정은 이 문서가 아니라 [DESIGN.md](DESIGN.md)의 결정 대장에
   기록한다 (D번호 부여).
4. 마일스톤 상태 변화는 보드와 DESIGN.md §3을 함께 갱신한다.
