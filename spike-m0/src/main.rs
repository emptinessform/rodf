//! rodf M0 스파이크 — 설계 문서(son31-main-design-20260823-114946.md)의 검증 항목.
//!
//! 판정 대상 (설계 문서 M0 기준):
//!   1. DOCX 관련 타입 import 0개로 rdocx 레이아웃 엔진을 구동할 수 있는가?
//!      → 정적 판정: 불가능. `rdocx_layout::LayoutInput`은 `CT_Document`,
//!        `CT_Styles` 등 rdocx-oxml(DOCX 모델) 타입을 필드로 직접 노출한다.
//!        이 파일이 rdocx_oxml을 import하는 것 자체가 실패 증거다.
//!   2. (폴백 경로 실측) ODF 모델 → DOCX 모델(CT_*) 매핑 어댑터의 비용:
//!      가상의 ODF 문단 + automatic style 1개를 손 매핑해 PNG까지 도달하는가?
//!   3. 스타일 매핑 미니 검증: ODF automatic style(글꼴/크기/굵기)이
//!      렌더 결과에 반영되는가? (크기가 다른 두 문단의 글리프 런 비교)

use std::collections::HashMap;

use rdocx_layout::{layout_document, LayoutInput, RevisionView};
use rdocx_oxml::{CT_Document, CT_P, CT_Styles, HalfPoint};

// ---------------------------------------------------------------------------
// 가상의 ODF 측 모델 — rodf-core가 content.xml/styles.xml 파싱 후 내놓을
// 형태의 축소판. (스파이크에서는 파싱 없이 손으로 만든다)
// ---------------------------------------------------------------------------

/// ODF automatic style을 flatten한 결과 (상속 해석 완료 가정).
struct OdfResolvedStyle {
    font_family: String,
    font_size_pt: u32,
    bold: bool,
}

struct OdfParagraph {
    text: String,
    style: OdfResolvedStyle,
}

// ---------------------------------------------------------------------------
// 어댑터: ODF 모델 → DOCX 모델(CT_*). rodf-render가 이 방향을 택할 경우의
// 최소 형태. DOCX-typed 구성물 사용 횟수를 세어 판정 데이터로 남긴다.
// ---------------------------------------------------------------------------

fn map_paragraph(odf: &OdfParagraph, docx_constructs: &mut Vec<&'static str>) -> CT_P {
    let mut p = CT_P::new();
    docx_constructs.push("CT_P");
    let run = p.add_run(&odf.text);
    docx_constructs.push("CT_R (via add_run)");
    let rpr = run.properties.get_or_insert_with(Default::default);
    docx_constructs.push("CT_RPr");
    rpr.font_ascii = Some(odf.style.font_family.clone());
    rpr.font_hansi = Some(odf.style.font_family.clone());
    rpr.font_east_asia = Some(odf.style.font_family.clone());
    rpr.sz = Some(HalfPoint(odf.style.font_size_pt * 2));
    docx_constructs.push("HalfPoint (DOCX 단위계)");
    if odf.style.bold {
        rpr.bold = Some(true);
    }
    p
}

fn main() {
    let mut docx_constructs: Vec<&'static str> = Vec::new();

    // ODF 쪽 입력: 크기가 다른 두 문단 (스타일 매핑 검증용)
    let paragraphs = [
        OdfParagraph {
            text: "안녕하세요 Hello — rodf M0 spike".to_string(),
            style: OdfResolvedStyle {
                font_family: "Malgun Gothic".to_string(),
                font_size_pt: 24,
                bold: true,
            },
        },
        OdfParagraph {
            text: "본문 크기 문단입니다. ODF automatic style이 여기 적용됩니다.".to_string(),
            style: OdfResolvedStyle {
                font_family: "Malgun Gothic".to_string(),
                font_size_pt: 11,
                bold: false,
            },
        },
    ];

    let mut doc = CT_Document::new();
    docx_constructs.push("CT_Document");
    for para in &paragraphs {
        doc.body.add_paragraph(map_paragraph(para, &mut docx_constructs));
    }

    let input = LayoutInput {
        revision_view: RevisionView::Accepted,
        document: doc,
        styles: CT_Styles::new_default(),
        numbering: None,
        headers: HashMap::new(),
        footers: HashMap::new(),
        images: HashMap::new(),
        charts: HashMap::new(),
        chart_theme: oxml_drawing::theme::CT_OfficeStyleSheet::office_default(),
        chart_color_map: oxml_drawing::color::ColorMap::default(),
        core_properties: None,
        hyperlink_urls: HashMap::new(),
        footnotes: None,
        endnotes: None,
        theme: None,
        fonts: Vec::new(),
    };
    docx_constructs.push("CT_Styles::new_default (DOCX 기본 스타일 테이블)");
    docx_constructs.push("LayoutInput (섹션/sectPr 기본값 내장)");

    let result = layout_document(&input).expect("layout failed");

    println!("=== rodf M0 spike 결과 ===");
    println!("pages: {}", result.pages.len());
    let elements: usize = result.pages.iter().map(|p| p.elements.len()).sum();
    println!("positioned elements: {elements}");

    let png = oxml_pdf::render_page_to_png(&result, 0, 144.0).expect("png render failed");
    std::fs::write("spike.png", &png).expect("write png");
    println!("spike.png written ({} bytes)", png.len());

    let pdf = oxml_pdf::render_to_pdf(&result);
    std::fs::write("spike.pdf", &pdf).expect("write pdf");
    println!("spike.pdf written ({} bytes)", pdf.len());

    println!();
    println!("=== 판정 데이터 ===");
    println!("[기준 1] DOCX 타입 import 0개: 실패 — 이 스파이크가 import한 DOCX 모델 타입:");
    for c in &docx_constructs {
        println!("  - {c}");
    }
    println!(
        "[기준 2] 어댑터 경로(ODF→CT_*) 실측: 문단+automatic style 매핑 {} 종의 DOCX 구성물 필요",
        {
            let mut kinds = docx_constructs.clone();
            kinds.sort();
            kinds.dedup();
            kinds.len()
        }
    );
    println!("[기준 3] 스타일 매핑: spike.png에서 두 문단의 크기/굵기 차이를 육안 확인할 것");
}
