# MCFS(Metric-Compatible Font Substitution) 라이선스 분석

> rodf의 핵심 방향인 MCFS(메트릭 호환 글꼴 대체) — 문서에 쓰인 폰트가 시스템에 없어도 유사한 모양을
> **원본과 동일한 메트릭**으로 대체해 레이아웃을 보존하는 기능 — 이
> 폰트 관련 법·라이선스와 어떤 관계에 있는지 정리한다.
>
> **이 문서는 법률 자문이 아니다.** 공개된 법리·판례·업계 선례를 근거로 한
> 설계 방침의 기록이며, 상용 배포 전 법률 검토를 대체하지 않는다.
>
> **용어**: 공식 명칭은 MCFS(메트릭 호환 글꼴 대체, 크레이트 `rmcf`). 유사 기능의
> 선행 사례인 한/글의 "FSL(Font Simulation Library)"은 한컴 고유 용어로,
> 본 프로젝트에서는 참조로만 언급한다.
>
> *(English summary at the end.)*

## 설계 원칙 — 무엇을 하고, 무엇을 하지 않는가

MCFS는 다음 네 가지 원칙 위에 설계된다:

1. **폰트 파일을 복제·내장·재배포하지 않는다.** 다루는 것은 숫자 메트릭
   (어드밴스 폭, ascender/descender, line gap)뿐이다.
2. **글리프 외형(outline)을 추출·복제하지 않는다.** 대체 글리프는 자유
   라이선스(OFL 등) 폰트에서 온다.
3. **산출물에 원본 폰트의 이름을 붙이지 않는다.** 원본 이름은 매핑 테이블의
   조회 키(지칭적 사용)로만 쓴다.
4. **기본 동작은 런타임 시뮬레이션이다.** 파일을 생성하지 않고, 사용자가
   정당하게 설치한 폰트를 렌더 시점에 활용한다.

## 법 영역별 분석

### 저작권 — 구조적으로 안전

- **한국**: 대법원은 서체 도안 자체의 저작물성을 부정한다 — 미적 요소가
  문자의 실용 기능에서 분리·독립되어 감상 대상이 될 정도가 아니라는 판단.
  보호되는 것은 **글자체를 디지털화한 폰트 파일**이며, 이는 컴퓨터프로그램
  저작물이다 (대법원 2001. 6. 29. 선고 99다23246 판결). MCFS는 파일을 복제하지
  않으므로 침해가 성립할 대상이 없다.
- **미국**: typeface 외형은 저작권 보호 대상이 아니다 (37 CFR 202.1(e),
  Eltra Corp. v. Ringer). 폰트 파일만 소프트웨어로 보호된다 — 한국과 같은
  이원 구조.
- **숫자 메트릭**: 순수 수치 데이터로, 저작권 보호 대상이 아니라는 것이
  통설이다.
- **업계 선례**: Liberation(Red Hat) · Croscore(Google)는 Arial/Times New
  Roman/Courier New의 메트릭을 그대로 재현한 폰트를 수십 년간 공개 배포해
  왔다. [PolarisOffice/polaris_mcfg](https://github.com/PolarisOffice/polaris_mcfg)
  (MIT)도 "글리프 외형을 추출/복제하지 않으며, 숫자 메트릭만 다룬다"를
  법적 안전 장치의 핵심으로 명시한다.

### 디자인보호법(한국 특유) — 해당 없음

2005년부터 글자체는 디자인 등록 대상이지만, 보호 대상은 **글리프 외형**이다.
MCFS는 외형을 자유 라이선스 폰트에서 가져오고 원본 외형을 복제하지 않는다.
또한 글자체 디자인권에는 통상적인 문서 작성·인쇄 과정의 사용과 그 결과물에
효력이 미치지 않는 제한 규정이 있다.

### 상표 — 원칙 3으로 회피

폰트 이름("맑은 고딕", "Arial" 등)은 상표일 수 있다. 문제가 되는 것은
**산출물에 원본 이름을 붙이는 행위**이고, 매핑 테이블에서 원본 이름을 조회
키로 참조하는 것(지칭적 사용)은 일반적으로 허용된다. 런타임 MCFS는 파일을
만들지 않으므로 이름을 붙일 산출물 자체가 없다.

## 잔여 리스크와 완화책

| # | 리스크 | 성격 | 완화책 |
|---|---|---|---|
| 1 | 상용 폰트 EULA의 리버스 엔지니어링/메트릭 추출 금지 조항 | 저작권과 별개인 **계약** 문제. "추출한 메트릭의 배포" 시점에 발생 | polaris_mcfg가 검증한 **렌더 기반 측정**(파일 파싱 없이 렌더 결과에서 측정, EULA-safe) 채택. 배포용 내장 테이블은 EULA 검토를 거친 폰트만 |
| 2 | OFL Reserved Font Name — OFL 폰트를 수정(메트릭 교체)해 배포하면 원래 이름 사용 금지 | OFL 조항 | 산출물 개명(mcfg 산출물 관행과 동일). 런타임 MCFS는 파일을 수정하지 않으므로 해당 없음 |
| 3 | 폰트 파일 자체의 취급 | 프로그램 저작물 | **절대 내장·재배포하지 않는다** (원칙 1) — 논쟁의 여지 자체를 제거 |

리스크의 무게중심: 대부분 "메트릭을 추출해 **배포**"하는 오프라인 생성 경로에
있다. rodf의 기본 축인 **런타임 시뮬레이션은 가장 안전한 형태**다 — 파일도
만들지 않고, 메트릭을 배포하지도 않으며, 사용자가 정당하게 설치한 폰트를
렌더 시점에 조정할 뿐이다.

## 운영 방침: 메트릭 출처 대장

배포용 내장 메트릭 테이블을 도입하는 시점(M2.5+)부터, 폰트별로 다음을
기록하는 출처 대장(`METRICS-PROVENANCE.md`)을 함께 유지한다:

- 대상 폰트와 버전
- 추출 방식: 테이블 파싱 / 렌더 기반 측정
- 추출 근거: 라이선스·EULA 검토 요지
- 추출 일자와 도구(예: polaris_mcfg 버전)

## 참고 자료

- [대한민국 정책브리핑 — 폰트 저작권 톺아보기](https://www.korea.kr/news/reporterView.do?newsId=148931087) (서체 도안 vs 폰트 파일 이원 보호)
- [PolarisOffice/polaris_mcfg](https://github.com/PolarisOffice/polaris_mcfg) — Metric-Compatible Font Generator (MIT)
- [Liberation fonts](https://en.wikipedia.org/wiki/Liberation_fonts) · [Croscore fonts](https://en.wikipedia.org/wiki/Croscore_fonts) — 메트릭 호환 폰트 선례
- [디자인보호법](https://www.law.go.kr/%EB%B2%95%EB%A0%B9/%EB%94%94%EC%9E%90%EC%9D%B8%EB%B3%B4%ED%98%B8%EB%B2%95) — 글자체 디자인 관련 조항

---

## English summary

rodf's MCFS (Metric-Compatible Font Substitution) substitutes missing document fonts with
similar-looking, **metric-identical** replacements so layout never shifts. Its
four design principles keep it clear of font licensing hazards: (1) never copy,
embed, or redistribute font files — only numeric metrics; (2) never extract
glyph outlines — replacement glyphs come from freely-licensed (OFL) fonts;
(3) never name outputs after the original font — original names are lookup keys
only (nominative use); (4) default to runtime simulation — no files are
generated, and legitimately installed fonts are adjusted at render time.

Legal basis: both Korean and US law protect font *files* (as software) but not
typeface *shapes* or numeric metrics; Korea's Supreme Court (99Da23246, 2001)
established this dual structure explicitly. Decades of precedent
(Liberation, Croscore) confirm metric-compatible substitution is accepted
practice. Residual risks are contractual (font EULAs restricting metric
extraction — mitigated by render-based measurement as validated by
polaris_mcfg) and naming (OFL Reserved Font Names — mitigated by renaming
generated artifacts; not applicable to runtime simulation). This document is a
design-policy record, not legal advice.
