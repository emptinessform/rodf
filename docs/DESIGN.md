# Design: rodf — 경량 고충실도 Rust ODF 라이브러리

Status: **APPROVED** · 최초 작성 2026-08-23 (office-hours 세션)
이 문서는 rodf의 설계·결정 기록의 **정본**이며 저장소에서 함께 관리한다.
최종 정리: 2026-08-23 (시간순 세션 기록을 현재 상태 기준으로 재구성)

---

## 1. 프로젝트 정의

### 문제와 포지션

rdoc(DOCX 뷰어/에디터) · rdocx(DOCX 라이브러리+레이아웃 엔진) · rhwp(HWP/HWPX
풀스택) 시리즈에 ODF 구성원이 없다. ODF 생태계의 빈 자리는 **"경량 고충실도
렌더링"**: LibreOffice 진영은 전체 스위트를 WASM으로 포팅 중(ZetaOffice, 수백
MB급)이고, 순수 Rust 진영(libreoffice-rs/lo_odf)은 생성 전용으로 레이아웃이
없다. rodf는 그 중간 — ODT를 파싱해 LibreOffice급 충실도로 PDF/PNG 렌더하는
작은 라이브러리. **한국어 타이포그래피 1급 요건.**

**EUREKA**: 모두가 "브라우저/임베디드 ODF = LibreOffice 통째 포팅"이라
가정하지만, rhwp/rdocx가 목적 특화 Rust 레이아웃 엔진으로 ~10MB WASM 충실도를
이미 증명했다. 이 가정의 붕괴가 rodf의 존재 이유다.

### 프로젝트의 세 가지 목적

1. **경량 고충실도 ODT 렌더링** — LibreOffice 오라클과의 SSIM 비교로 검증
   (M1 달성 완료).
2. **MCFS(메트릭 호환 글꼴 대체)** — 폰트가 없어도 레이아웃이 흐트러지지 않는
   국제 표준 기반 크로스 포맷 폰트 계층 (→ §4).
3. **통합 뷰어/에디터의 기반** — doc/docx·hwp/hwpx·odf 모든 문서를 한 곳에서
   보고 편집하는 구조의 수렴점 (→ §5).

### 제약

- 오픈소스/커뮤니티 프로젝트 (D1). 라이선스 MIT OR Apache-2.0 (rdocx 관례).
- 순수 Rust. rodf-core는 zip+quick-xml만; 렌더링 의존성은 엔진 경유 상속.
- 지원 범위(초기): ODF 1.2+ 표준 ZIP 패키지의 ODT만 — flat ODF/암호화/손상
  파일은 명시적 에러.
- rdocx 포크의 Word 경로 동작을 해치지 않을 것 (font-natural sentinel 방식).

---

## 2. 아키텍처 (현재)

```
 rodf-core        ODF 패키지 + 문서 모델 + 스타일 해석 (zip, quick-xml만)
    │                automatic/named/default-style 체인 flatten,
    │                서양(fo:*)/동아시아(style:*-asian) 속성 분리,
    │                master-page 페이지 기하
    ▼
 rodf-render      ODF 모델 → 중립 IR 매핑 (문자체계별 런 분할)
    ▼
 rlayout          ★ 중립 문서 IR + 플로우 엔진 (M2, 자체 크레이트)
    │                LO/ODF 관례 기본값: 폰트 자연 행간(gap 위),
    │                한글 어절 줄바꿈 — sentinel 불필요
    ▼
 oxml-layout      포맷 중립 공유 계층: 폰트/셰이핑/줄바꿈/LayoutResult
    ▼                (포크 모노레포 소속, rev 고정 — DOCX 모델은 미의존)
 oxml-pdf         LayoutResult → PDF/PNG (중립)

 rodf-cli         rodf render in.odt out.{pdf,png}
 tools/oracle.py  LO 오라클: crop 정규화 → ±2px 정합 → blur2 SSIM ≥ 0.95
 tools/corpus.py  코퍼스 스코어보드 (게이트 아님) → docs/scoreboard.{md,json}
```

- **M2 완료 (2026-08-23)**: 경로 α(ODF→CT_* 어댑터)를 rlayout 전환으로 대체 —
  D10 하이브리드가 예정한 β 전환의 실현. rodf 의존 그래프에서
  rdocx-layout/rdocx-oxml 소멸, font-natural sentinel도 rodf 경로에서 불필요
  (포크의 sentinel은 Word 크레이트 쪽 호환용으로 잔존). 스코어보드 10/10 유지로
  픽셀 등가 검증. MappingLoss API는 IR 미표현 의미론 기록용으로 유지(현재 0건).
- **엔진 의존**: `emptinessform/rdocx` git 의존, rev `7575436` 고정. 포크는
  자체 발전하는 독립 라인 — 업스트림(tensorbee) PR 보류 (→ §6).

---

## 3. 현재 상태 (2026-08-23 기준)

| 마일스톤 | 상태 | 근거 |
|---|---|---|
| M0 엔진 분리성 스파이크 | **완료** | 엄격 중립성 실패 확인, 어댑터 저비용 실증 → 부록 A.1 |
| M1 파싱→렌더→오라클 ≥0.95 | **완료** | hello blur2 0.9829 / gulim 0.9764 PASS → 부록 A.2~A.4 |
| M1.5 코퍼스+CI | **1차 슬라이스 완료** | 코퍼스 10문서, 스코어보드 6/10 PASS, GitHub Actions |
| M2 rlayout 승격 | **완료** (D13) | rlayout v0 + rodf 전환, 스코어보드 10/10 유지, DOCX 의존 소멸 |

- 테스트 **20개** (전부 테스트 우선 작성): core 5 · render 12 · cli 3.
- 저장소 공개: README(한/영 병기) + side-by-side 이미지 + MCFS 라이선스 문서.

### 백로그 — **전부 해소 (2026-08-23)**: 스코어보드 10/10 PASS

1. ~~합성 기울임 (0.69)~~ → 20°(DirectWrite 관례) 스큐 합성, 0.97. 포크 1a7a5db.
2. ~~한글 줄바꿈 규칙 (0.84)~~ → font-natural 모드에 어절 단위(hangul_word_wrap),
   줄바꿈 위치 LO와 완전 일치. 포크 dbc756f.
3. ~~multi-paragraph (0.90)~~ → 1+2 해결로 0.96.
4. ~~AA/힌팅 잔여 (wrap-korean 0.947, batang 0.949)~~ → **오라클 v2**로 규명:
   양쪽 PDF를 동일 래스터라이저(pdftoppm)로 비교하면 raw 0.99+ — 잔여는 전부
   LO 래스터(GDI/DWrite 감마·힌팅) vs tiny-skia 차이. 판정은 pdf 경로, png
   경로는 참고 열. rodf 자체 래스터의 감마 근사는 선택 과제로 대기열에.

### M1.5 잔여 과제

- LO 버전 고정 Docker + 고정 폰트 오라클 (세션 간 비결정성 실측으로 근거 확립).
- 공개 라이선스 ODT 수집 (출처·저작권 기록, 50~100개 규모).
- CI에서 렌더 테스트 안정화 (러너 한국어 폰트 문제 — OFL 폰트 번들 검토).

---

## 4. 핵심 차별 기능: MCFS — Metric-Compatible Font Substitution

> **용어 (D12)**: 공식 명칭 **MCFS(메트릭 호환 글꼴 대체)**, 시리즈 공용
> 크레이트명 **`rmcf`**. "FSL(Font Simulation Library)"은 한/글(한컴) 고유
> 용어로 선행 사례 참조로만 사용. 용어 계보: 업계 기성 용어 "metric-compatible
> fonts"(Liberation/Croscore/polaris_mcfg) + 런타임 메커니즘은 W3C CSS Fonts의
> font metric override(size-adjust/ascent-override 등)와 대응.

**목표**: 문서에 사용된 폰트가 시스템에 없어도 유사한 모양 + 동일한 메트릭의
대체를 제공해 레이아웃(줄바꿈·쪽나눔)을 보존한다. 한/글의 사설 방식이 아닌
**국제 표준 기반**, rodf 전용이 아닌 **ODF·HWP/HWPX·DOC/DOCX 공용 구조**.

**표준 근거** — 세 포맷 모두 폰트 기술자를 이미 싣고 다닌다:
- ODF: `style:font-face`의 `svg:panose-1`, `style:font-family-generic`, `style:font-pitch`
- DOCX: `fontTable.xml`의 `w:panose1`, `w:charset`, `w:family`, `w:pitch`, `w:sig`
- HWP/HWPX: FaceName 레코드의 대체 글꼴 이름 + 글꼴 유형 정보(PANOSE형 분류)
- 매칭 축: PANOSE-1 (OpenType OS/2, ISO/IEC 14496-22) — fontdb/ttf-parser로 판독.

**파이프라인**: ① 기술자 추출(포맷별 파서 → 공통 FontDescriptor) → ② 후보
선정(PANOSE 거리 + 한글 커버리지 + generic 폴백) → ③ 메트릭 시뮬레이션(유명
한국어 폰트 어드밴스 테이블 내장, 글리프 폭 스케일 → 줄바꿈 불변; 보조:
size-adjust식 정규화, 합성 볼드/이탤릭). **실측으로 추가된 범위**: 한/영 폰트명
별칭 해석(부록 A.3의 "Malgun Gothic"/"맑은 고딕" 사건).

**아키텍처 위치**: oxml-layout 폰트 경로 옆의 공유 계층, rlayout(M2)과 정합.
1단계(기술자+PANOSE 매칭)는 M2 후보, 메트릭 내장 테이블은 M2.5+.

**라이선스**: 상세 분석은 저장소 공개 문서
[docs/mcfs-licensing.md](https://github.com/emptinessform/rodf/blob/main/docs/mcfs-licensing.md).
요지 — 4원칙(파일 비복제·외형 비추출·원본명 비사용·런타임 우선) 하에서
저작권(한국 대법원 99다23246 이원 구조, 미 37 CFR 202.1(e))·디자인보호법·
상표 문제가 구조적으로 없음. 잔여 리스크는 EULA(→ 렌더 기반 측정)와 OFL
RFN(→ 개명). 배포용 메트릭 테이블 도입 시 METRICS-PROVENANCE 출처 대장 유지.

**선행 오픈소스**: [PolarisOffice/polaris_mcfg](https://github.com/PolarisOffice/polaris_mcfg)
(MIT) — 메트릭만 추출해 자유 폰트 글리프에 입혀 대체 TTF를 오프라인 생성.
rmcf(런타임)와 상호보완, 메트릭 JSON 스펙을 rmcf 데이터 포맷으로 채택 검토.
같은 조직의 polaris_dvc(Hancom DVC Rust 포팅)도 인접 프로젝트로 주시.

---

## 5. 확장 목적: 통합 뷰어/에디터

**목적**: rodf의 에디터/뷰어 계층은 ODF 전용이 아니라, doc/docx·hwp/hwpx·odf
**모든 문서를 한 곳에서 보고(뷰어) 편집하는 통합 구조**의 기반. rdoc·rhwp가
각자 증명한 계층의 수렴점.

**구조 결정 (D11): 멀티 백엔드 뷰어 셸 먼저.**
- 통합 뷰어 = 포맷별 네이티브 모델(rdocx/rhwp/rodf 파서) + 공유 렌더
  (LayoutResult → 공통 셸) — 변환 손실 없음. 하부 구조는 이미 절반 존재.
- 편집 구조(네이티브 vs ODF 피벗)는 뷰어 운영 경험 후 결정. ODF 허브(변환)
  방식은 손실이 구조적 한계라 기각.
- 판단 기준: 모든 설계 결정에서 **중립 계층(rlayout/rmcf) 강화 > 포맷 종속
  지름길**. 통합 뷰어 셸 자체는 M3+ 이후 배치.

---

## 6. 확정된 결정 대장

| # | 결정 | 내용 |
|---|---|---|
| D1 | 프로젝트 성격 | 오픈소스/커뮤니티 (추천이던 제품 라인 대신 사용자 선택) |
| D2 | 목표 계층 | 라이브러리+레이아웃/렌더링 (rdocx 패턴), ODT 우선 |
| D3 | 첫 데모 | ODT→PDF/PNG, LibreOffice side-by-side 한 장 |
| D5·D7 | 전제 확정·수정 | "엔진 재사용 가능"을 가정에서 **M0 검증 스파이크**로 강등 (2차 의견 반영) |
| D8 | 접근 | A(스파이크 우선 수직 슬라이스) + 조건부 B(rlayout=M2)/C(오라클→대시보드=M1.5/M2.5) 흡수 |
| D9 | 설계 승인 | 3라운드 적대적 리뷰(이슈 18건 수정) 후 APPROVED |
| D10 | M1 경로 | 하이브리드 — 어댑터(α)로 진행 + 매핑 손실 수집으로 β 전환 판단 |
| D11 | 통합 뷰어 구조 | 멀티 백엔드 뷰어 셸 먼저, 편집 구조는 경험 후 결정 |
| D12 | 용어 | MCFS / 크레이트 `rmcf` ("FSL"은 한/글 용어, 참조만) |
| — | 저장소·라이선스 | emptinessform/rodf 공개, MIT OR Apache-2.0 듀얼 |
| D13 | M2 착수 방향 | 기능 확장(M3)보다 **rlayout 승격 우선** (2026-08-23). 전략: rdocx-layout에서 추출하지 않고 **rodf 워크스페이스에 `crates/rlayout` 신규 작성**(rpptx 선례) — oxml-layout 위의 중립 문서 IR + 플로우 엔진, LO 관례가 기본값(font-natural sentinel·MappingLoss 소멸). 완료 기준: 스코어보드 10/10 유지 + rodf 의존에서 rdocx-oxml/rdocx-layout 제거. 안정 후 시리즈 공용 위치로 이동. 포크 동시 세션 충돌 회피 겸용 |
| D14 | ODF 텍스트 모델 실측 3건 (2026-08-23, space.odt 오라클) | ① **공백 병합**: 문자 데이터의 연속 공백은 스팬 경계를 넘어 1개로 병합, 문단 선두 공백 제거, `text:s`는 무조건 방출+병합 상태 리셋(뒤 리터럴 공백 생존). ② **LO 내장 Heading**: 스타일 미정의 text:h는 Liberation Sans 14pt×배율(H1 130%)·bold·위 0.42cm/아래 0.21cm — 문서 정의가 항상 이김(default ← builtin ← chain 순). ③ **LO 프로그램 기본 탭 간격 = 2cm**(1.25cm 아님 — LO 저장 문서가 1.25cm를 명시할 뿐). 부수 결함 2건 수정: quick-xml 0.41 GeneralRef(엔티티) 소실, self-closing paragraph-properties의 tab-stop-distance 미파싱(폴백=문서값 우연 일치로 은폐돼 있었음) |
| — | 업스트림 관계 | **독립 라인 유지, tensorbee PR 보류.** 포크(svg-poc-0.8)는 자체 커밋이 쌓인 독립 엔진 라인 — PR 하나로 포크 의존이 해소되지 않아 실익 없음. 재검토 조건: ① 업스트림 추종 전략 전환 시 ② 기여 자체가 목적일 때 |

---

## 7. Open Questions (현행)

- ODF 스타일 상속 매핑 손실의 전체 목록 — 기본 속성(글꼴/크기/굵기/기하/행간)은
  무손실 확인, master-page 다중 스타일·표 모델 등은 해당 기능 구현 시 수집.
- lo_odf(libreoffice-rs)와의 관계 — 경쟁/무시/협력(직렬화 참고) 미정.
- CI 렌더 테스트의 폰트 전략 — OFL 폰트 번들 vs 러너 폰트 설치.
- 경로 β 전환 시점 — MappingLoss 누적과 백로그 처리 경험으로 판단.

## 8. Success Criteria (현행)

- ~~M0·M1~~ **달성** (부록 A). 오라클 판정 기준 확정: crop 정규화 → ±2px 정합 →
  콘텐츠 bbox 크롭 → **blur2 SSIM ≥ 0.95**.
- M1.5: 결정론 확보된 오라클이 코퍼스 50+ 문서에 대해 CI에서 flaky 없이 동작
  (스코어보드 방식: 크래시 0 + 점수 결정론).
- 커뮤니티: side-by-side 공개 후 첫 외부 이슈/스타 유입.

## 9. Distribution Plan

- GitHub 공개(완료) → crates.io (`rodf-core`/`rodf-render`/`rodf-cli`) → npm
  WASM (`@rodf/core` 스타일, 시리즈 관례).
- CI: GitHub Actions(가동) + LO 고정 Docker 오라클(예정).

---

## 부록 A. 세션 기록 (2026-08-23, 시간순 요약)

### A.1 M0 스파이크

위치 `D:\sb\SBOdf\spike-m0`. **기준 1(DOCX import 0) 실패** — LayoutInput이
CT_Document/CT_Styles를 시그니처에 노출. **기준 2(어댑터 비용) 통과** — 문단+
automatic style 매핑에 DOCX 구성물 7종, ~150줄로 PNG/PDF 출력. **기준 3(스타일
매핑) 통과.** **핵심 발견**: oxml-layout(4.9K줄)이 이미 포맷 중립 기반이고
rdocx-layout(14.8K)/rpptx-layout(8K)이 포맷별 플로우 엔진 — **rlayout의 씨앗은
신규 추출이 아니라 oxml-layout의 확장.**

### A.2 M1 구현 (TDD)

rodf-core(패키지/파서/스타일 flatten/서양·asian 분리/페이지 기하) +
rodf-render(to_layout_input, 문자체계별 런 분할, sectPr 매핑, MappingLoss) +
rodf-cli + oracle.py. 오라클이 잡은 이슈 3건 수정: ① 페이지 기하 미매핑
(US Letter로 렌더) ② asian 속성 무시(한글이 24pt로 렌더 — ODF의
fo:*/style:*-asian 분리는 한국어 1급 요건의 핵심) ③ Word Normal 행간 누출.

### A.3 행간 모델 실험 — 진단 정정의 기록

- 최초 진단 "엔진이 Word처럼 행을 계산(β 증거 1호)"은 **철회**: 통제 픽스처에서
  엔진과 LO가 픽셀 일치. **LO 행 모델 = hhea ascent+|descent|+lineGap** (맑은
  고딕 1.33em, Arial 1.15em 검증), 혼합 크기 행은 최대 포션 지배.
- **진짜 원인**: 픽스처의 영문 폰트명 "Malgun Gothic"을 LO가 미해석 → 라틴
  포션 Arial 폴백(1.15em). rodf(fontdb)는 영문명 매칭 성공 → 4px 차이.
  **폰트명 한/영 별칭 해석을 MCFS 범위에 추가** (실측된 충실도 위험).
- **lineGap>0 재검증 → 실결함 수정**: 굴림/바탕(gap 152/1024)에서 엔진 1.0em vs
  LO 1.148em(행당 15% 오차), LO는 gap을 행 위에 배치. 원인: 엔진이 세그먼트에
  line_gap=0 하드코딩 + Word 재현 패스가 높이 덮어씀. 수정: 포크에
  `"font-natural"` line_rule sentinel(4ab9020, Word 경로 불변 — 147 테스트
  통과), rodf 전 문단 적용, 굴림 픽셀 테스트 2건 고정.

### A.4 오라클 방법론 확정 — M1 달성

측정 버그 2건 수정: ① LO PNG 크기 반올림의 세션 간 ±1px 요동을 resize로
정규화해 전체 흐려짐 → **crop 정규화** ② 여백 반올림의 전역 1px 오프셋 감점 →
**±2px 정합**. 결과: **hello blur2 0.9829 (raw 0.9211), gulim 0.9764 — PASS.**
LO는 세션 내 결정론/세션 간 비결정(폰트 캐시) — Docker 고정 근거.

### A.5 M1.5 1차 슬라이스

코퍼스 10문서(gen_corpus.py) + 스코어보드(corpus.py) 6/10 PASS + GitHub
Actions. 실패 4건은 §3 백로그로.

## 부록 B. Office-hours 원본 기록 (요약)

- **검토된 대안**: A 스파이크 우선 수직 슬라이스(선택) / B rlayout 승격
  ("문서 포맷의 LLVM", XL·High) / C 오라클 주도 대시보드 우선(M) — A+조건부
  B/C 흡수로 확정.
- **2차 의견(독립 콜드 리딩) 핵심**: rlayout 승격 비전 제안, "rdoc은 엔진
  중립성의 근거가 못 된다"는 전제 1 반박(수용), 주말 200줄 스파이크 설계 제안
  (그대로 실행됨), 공개 충실도 대시보드 아이디어(M2.5 채택).
- **The Assignment**: "주말에 200줄로 전제 1을 검증하라" — 당일 실행 완료.
- **What I noticed**: 시리즈 관점 사고 / 경쟁 지형 우선 정의 / 권고를 맥락으로
  덮어쓰는 판단(D1) / 애착보다 증거(D7 수용).
