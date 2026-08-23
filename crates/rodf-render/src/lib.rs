//! rodf-render — ODF 문서 모델을 rdocx 레이아웃 엔진 입력으로 매핑(경로 α)하고
//! PDF/PNG를 생성한다. DOCX 모델이 표현하지 못하는 ODF 의미론은
//! [`MappingLoss`]로 수집한다 — 이 목록이 경로 β 전환 판단의 데이터가 된다.

use std::collections::HashMap;

use rdocx_layout::{layout_document, LayoutInput, RevisionView};
use rdocx_oxml::{CT_Document, CT_P, CT_SectPr, CT_Styles, HalfPoint, Twips};
use rodf_core::Document;

/// 어댑터가 보존하지 못한 ODF 의미론 하나.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MappingLoss {
    /// 무엇을 잃었는가 (예: "master-page").
    pub what: String,
    /// 상세 (예: 스타일 이름, 속성 값).
    pub detail: String,
}

#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    #[error("layout error: {0}")]
    Layout(String),
}

/// 레이아웃 완료된 문서. 페이지 단위 렌더와 손실 목록을 노출한다.
pub struct Rendered {
    layout: oxml_layout::LayoutResult,
    losses: Vec<MappingLoss>,
}

impl Rendered {
    pub fn page_count(&self) -> usize {
        self.layout.pages.len()
    }

    pub fn page_png(&self, page_index: usize, dpi: f64) -> Option<Vec<u8>> {
        oxml_pdf::render_page_to_png(&self.layout, page_index, dpi)
    }

    pub fn pdf(&self) -> Vec<u8> {
        oxml_pdf::render_to_pdf(&self.layout)
    }

    pub fn losses(&self) -> &[MappingLoss] {
        &self.losses
    }
}

/// ODF 문서를 rdocx 엔진 입력으로 매핑한다 (경로 α 어댑터).
/// 렌더 전 입력을 검사·수정하려는 호출자를 위해 공개한다.
pub fn to_layout_input(doc: &Document) -> (LayoutInput, Vec<MappingLoss>) {
    let mut losses = Vec::new();

    let mut docx = CT_Document::new();
    for paragraph in doc.paragraphs() {
        docx.body.add_paragraph(map_paragraph(paragraph, &mut losses));
    }
    if let Some(geometry) = doc.page_geometry() {
        docx.body.sect_pr = Some(map_page_geometry(geometry));
    }

    let input = LayoutInput {
        revision_view: RevisionView::Accepted,
        document: docx,
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
    (input, losses)
}

/// ODF 문서를 rdocx 엔진으로 레이아웃한다 (경로 α 어댑터).
pub fn render(doc: &Document) -> Result<Rendered, RenderError> {
    let (input, losses) = to_layout_input(doc);
    let layout = layout_document(&input).map_err(|e| RenderError::Layout(e.to_string()))?;
    Ok(Rendered { layout, losses })
}

/// ODF master-page 기하를 DOCX 섹션 속성으로 매핑한다 (pt → twips).
fn map_page_geometry(geometry: &rodf_core::PageGeometry) -> CT_SectPr {
    let twips = |pt: f64| Twips((pt * 20.0).round() as i32);
    let mut sect_pr = CT_SectPr::default_letter();
    sect_pr.page_width = Some(twips(geometry.width_pt));
    sect_pr.page_height = Some(twips(geometry.height_pt));
    sect_pr.margin_top = Some(twips(geometry.margin_top_pt));
    sect_pr.margin_right = Some(twips(geometry.margin_right_pt));
    sect_pr.margin_bottom = Some(twips(geometry.margin_bottom_pt));
    sect_pr.margin_left = Some(twips(geometry.margin_left_pt));
    sect_pr
}

/// 문자 체계 — ODF의 서양(fo:*) / 동아시아(style:*-asian) 속성 구분에 대응.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Script {
    Western,
    Asian,
}

fn classify(c: char) -> Option<Script> {
    if c.is_whitespace() {
        return None; // 중립 — 인접 강한 문자에 붙는다.
    }
    let cp = c as u32;
    let asian = matches!(cp,
        0x1100..=0x11FF   // Hangul Jamo
        | 0x3000..=0x303F // CJK Symbols and Punctuation
        | 0x3040..=0x30FF // Hiragana, Katakana
        | 0x3130..=0x318F // Hangul Compatibility Jamo
        | 0x31F0..=0x31FF // Katakana Phonetic Extensions
        | 0x3400..=0x4DBF // CJK Ext A
        | 0x4E00..=0x9FFF // CJK Unified Ideographs
        | 0xA960..=0xA97F // Hangul Jamo Extended-A
        | 0xAC00..=0xD7FF // Hangul Syllables + Jamo Extended-B
        | 0xF900..=0xFAFF // CJK Compatibility Ideographs
        | 0xFF00..=0xFFEF // Halfwidth and Fullwidth Forms
    );
    Some(if asian { Script::Asian } else { Script::Western })
}

/// 텍스트를 문자 체계별 연속 런으로 나눈다. 공백 등 중립 문자는
/// 직전 런에 붙고, 선행 중립 문자는 첫 강한 문자의 런에 합류한다.
pub fn split_script_runs(text: &str) -> Vec<(Script, String)> {
    let mut runs: Vec<(Script, String)> = Vec::new();
    let mut pending_neutral = String::new();

    for c in text.chars() {
        match classify(c) {
            None => pending_neutral.push(c),
            Some(script) => {
                match runs.last_mut() {
                    Some((last, buf)) if *last == script => {
                        buf.push_str(&pending_neutral);
                        buf.push(c);
                    }
                    Some((_, buf)) => {
                        // 중립 문자는 직전 런에 남긴다.
                        buf.push_str(&pending_neutral);
                        runs.push((script, c.to_string()));
                    }
                    None => {
                        let mut s = std::mem::take(&mut pending_neutral);
                        s.push(c);
                        runs.push((script, s));
                    }
                }
                pending_neutral.clear();
            }
        }
    }
    if !pending_neutral.is_empty() {
        if let Some((_, buf)) = runs.last_mut() {
            buf.push_str(&pending_neutral);
        }
        // 강한 문자가 하나도 없으면(공백뿐) 런 없음 — 문단은 상위에서 처리.
    }
    runs
}

/// ODF 문단(해석 완료 스타일 포함)을 DOCX 문단으로 매핑한다.
///
/// ODF 스타일 상속은 rodf-core에서 이미 flatten되어 있고, 서양/동아시아
/// 속성 분리는 DOCX 단일 w:sz로 표현할 수 없으므로 문자 체계별로 런을
/// 나눠 각 런에 해당 속성을 적용한다.
fn map_paragraph(paragraph: &rodf_core::Paragraph, losses: &mut Vec<MappingLoss>) -> CT_P {
    let mut p = CT_P::new();
    let style = paragraph.style();

    // ODF/LO 기본은 문단 간격 0·단일 행간 — Word Normal 기본값(1.08 행간,
    // after-spacing)이 새어들지 않도록 명시적으로 고정한다.
    {
        let ppr = p.properties.get_or_insert_with(Default::default);
        ppr.space_before = Some(Twips(0));
        ppr.space_after = Some(Twips(0));
        ppr.line_spacing = Some(Twips(240));
        ppr.line_rule = Some("auto".to_string());
    }

    for (script, segment) in split_script_runs(paragraph.text()) {
        let run = p.add_run(&segment);
        let rpr = run.properties.get_or_insert_with(Default::default);

        let (family, size_pt, bold, italic) = match script {
            Script::Western => (
                style.font_family.clone(),
                style.font_size_pt,
                style.bold,
                style.italic,
            ),
            Script::Asian => (
                style
                    .font_family_asian
                    .clone()
                    .or_else(|| style.font_family.clone()),
                style.font_size_asian_pt.or(style.font_size_pt),
                style.bold_asian,
                style.italic_asian,
            ),
        };

        if let Some(family) = family {
            rpr.font_ascii = Some(family.clone());
            rpr.font_hansi = Some(family.clone());
            rpr.font_east_asia = Some(family);
        }
        match size_pt {
            Some(size_pt) if size_pt > 0.0 => {
                let half_points = (size_pt * 2.0).round();
                if (half_points - size_pt * 2.0).abs() > f64::EPSILON {
                    losses.push(MappingLoss {
                        what: "font-size-precision".to_string(),
                        detail: format!("{size_pt}pt rounded to {} half-points", half_points),
                    });
                }
                rpr.sz = Some(HalfPoint(half_points as u32));
            }
            Some(size_pt) => losses.push(MappingLoss {
                what: "font-size-invalid".to_string(),
                detail: format!("{size_pt}pt dropped"),
            }),
            None => {}
        }
        if bold {
            rpr.bold = Some(true);
        }
        if italic {
            rpr.italic = Some(true);
        }
    }
    p
}
