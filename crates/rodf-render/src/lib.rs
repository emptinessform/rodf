//! rodf-render — ODF 문서 모델을 중립 IR(rlayout)로 매핑해 배치하고
//! PDF/PNG를 생성한다 (M2, D13: 경로 α의 DOCX 모델 어댑터를 대체).
//!
//! rlayout은 LO/ODF 관례(폰트 자연 행간, gap 위 배치, 한글 어절 줄바꿈)가
//! 기본값이므로 Word 에뮬레이션을 우회하는 sentinel이 필요 없다.
//! [`MappingLoss`]는 IR이 표현하지 못하는 ODF 의미론을 만났을 때 기록하는
//! 자리로 유지된다 (네이티브 IR 전환으로 현재는 발생 항목 없음).

use rodf_core::Document;

/// 렌더 경로가 보존하지 못한 ODF 의미론 하나.
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

/// A4 폴백 (문서에 master-page가 없을 때).
const FALLBACK_PAGE: rlayout::PageGeometry = rlayout::PageGeometry {
    width_pt: 595.3,
    height_pt: 841.9,
    margin_top_pt: 56.7,
    margin_right_pt: 56.7,
    margin_bottom_pt: 56.7,
    margin_left_pt: 56.7,
};

/// ODF 문서를 중립 IR로 매핑한다. 렌더 전 IR을 검사하려는 호출자를 위해 공개.
pub fn to_document(doc: &Document) -> (rlayout::Document, Vec<MappingLoss>) {
    let losses = Vec::new();

    let page = doc
        .page_geometry()
        .map(|g| rlayout::PageGeometry {
            width_pt: g.width_pt,
            height_pt: g.height_pt,
            margin_top_pt: g.margin_top_pt,
            margin_right_pt: g.margin_right_pt,
            margin_bottom_pt: g.margin_bottom_pt,
            margin_left_pt: g.margin_left_pt,
        })
        .unwrap_or(FALLBACK_PAGE);

    let blocks = doc
        .paragraphs()
        .iter()
        .map(|paragraph| rlayout::Block::Paragraph(map_paragraph(paragraph)))
        .collect();

    (rlayout::Document { page, blocks }, losses)
}

/// ODF 문서를 배치한다.
pub fn render(doc: &Document) -> Result<Rendered, RenderError> {
    let (ir, losses) = to_document(doc);
    let layout = rlayout::layout(&ir).map_err(|e| RenderError::Layout(e.to_string()))?;
    Ok(Rendered { layout, losses })
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
    }
    runs
}

/// ODF 문단(해석 완료 스타일 포함)을 IR 문단으로 매핑한다.
///
/// ODF의 서양/동아시아 속성 분리는 문자 체계별 런 분할로 표현한다 —
/// 각 런이 해당 체계의 글꼴·크기·굵기를 갖는다.
fn map_paragraph(paragraph: &rodf_core::Paragraph) -> rlayout::Paragraph {
    let style = paragraph.style();
    let runs = split_script_runs(paragraph.text())
        .into_iter()
        .map(|(script, segment)| {
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
            rlayout::Run {
                text: segment,
                style: rlayout::TextStyle {
                    font_family: family,
                    font_size_pt: size_pt.unwrap_or(12.0),
                    bold,
                    italic,
                },
            }
        })
        .collect();

    rlayout::Paragraph {
        runs,
        align: match style.align {
            Some(rodf_core::Align::Center) => rlayout::Align::Center,
            Some(rodf_core::Align::End) => rlayout::Align::End,
            Some(rodf_core::Align::Justify) => rlayout::Align::Justify,
            Some(rodf_core::Align::Start) | None => rlayout::Align::Start,
        },
        space_before_pt: 0.0,
        space_after_pt: 0.0,
        hangul_word_wrap: None, // 기본값(true) = LO/ODF 관례
    }
}
