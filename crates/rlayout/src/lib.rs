//! rlayout — 포맷 중립 문서 IR + 플로우 레이아웃 엔진 (M2, D13).
//!
//! ODF/HWP/DOCX 등 포맷별 프런트엔드가 이 IR로 문서를 기술하면,
//! oxml-layout(폰트/셰이핑/줄바꿈/출력 모델) 위에서 페이지를 배치해
//! `LayoutResult`를 돌려준다. 규칙 기본값은 폰트 메트릭 관례를 따른다:
//! 행 높이 = hhea ascent+descent+lineGap(gap은 행 위), 한글은 어절 단위
//! 줄바꿈 — Word 에뮬레이션 sentinel 없이 이것이 기본값이다.

use oxml_layout::{
    Color, FontManager, InlineItem, LayoutResult, LineBreakParams, LineItem, LineSpacing,
    PageFrame, Point, PositionedElement, TextSegment,
};

/// 페이지 기하 (pt).
#[derive(Debug, Clone, PartialEq)]
pub struct PageGeometry {
    pub width_pt: f64,
    pub height_pt: f64,
    pub margin_top_pt: f64,
    pub margin_right_pt: f64,
    pub margin_bottom_pt: f64,
    pub margin_left_pt: f64,
}

/// 텍스트 런의 해석 완료된 스타일. 상속/자동 스타일 해석은 프런트엔드 몫.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TextStyle {
    pub font_family: Option<String>,
    pub font_size_pt: f64,
    pub bold: bool,
    pub italic: bool,
}

#[derive(Debug, Clone)]
pub struct Run {
    pub text: String,
    pub style: TextStyle,
}

/// 문단 정렬.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Align {
    #[default]
    Start,
    Center,
    End,
    Justify,
}

#[derive(Debug, Clone, Default)]
pub struct Paragraph {
    pub runs: Vec<Run>,
    pub align: Align,
    pub space_before_pt: f64,
    pub space_after_pt: f64,
    /// 한글 어절 단위 줄바꿈 (기본 true — LibreOffice/ODF 관례).
    pub hangul_word_wrap: Option<bool>,
}

#[derive(Debug, Clone)]
pub enum Block {
    Paragraph(Paragraph),
}

/// 중립 문서 IR — 레이아웃 엔진의 입력.
#[derive(Debug, Clone)]
pub struct Document {
    pub page: PageGeometry,
    pub blocks: Vec<Block>,
}

#[derive(Debug, thiserror::Error)]
pub enum RlayoutError {
    #[error("layout error: {0}")]
    Layout(#[from] oxml_layout::LayoutError),
}

/// 문서를 배치해 렌더 가능한 결과를 만든다.
pub fn layout(doc: &Document) -> Result<LayoutResult, RlayoutError> {
    let mut fm = FontManager::new();
    layout_with_fonts(doc, &mut fm)
}

/// 호출자가 준비한 FontManager로 배치한다 (재사용/결정론 폰트 셋 용).
pub fn layout_with_fonts(
    doc: &Document,
    fm: &mut FontManager,
) -> Result<LayoutResult, RlayoutError> {
    let page = &doc.page;
    let content_width = page.width_pt - page.margin_left_pt - page.margin_right_pt;
    let content_bottom = page.height_pt - page.margin_bottom_pt;

    let mut pages: Vec<PageFrame> = Vec::new();
    let mut elements: Vec<PositionedElement> = Vec::new();
    let mut cursor_y = page.margin_top_pt;
    let mut page_number = 1usize;

    let mut flush_page =
        |elements: &mut Vec<PositionedElement>, pages: &mut Vec<PageFrame>, n: usize| {
            pages.push(PageFrame::new(
                n,
                page.width_pt,
                page.height_pt,
                std::mem::take(elements),
            ));
        };

    for block in &doc.blocks {
        let Block::Paragraph(paragraph) = block;

        // 런 셰이핑 → 중립 세그먼트
        let mut items: Vec<InlineItem> = Vec::new();
        for run in &paragraph.runs {
            if run.text.is_empty() {
                continue;
            }
            let style = &run.style;
            let font_id = fm.resolve_font_for_text(
                style.font_family.as_deref(),
                style.bold,
                style.italic,
                &run.text,
            )?;
            let metrics = fm.metrics(font_id, style.font_size_pt)?;
            let shaped = fm.shape_text(font_id, &run.text, style.font_size_pt)?;
            items.push(InlineItem::Text(TextSegment {
                text: run.text.clone(),
                source: None,
                font_id,
                font_size: style.font_size_pt,
                glyph_ids: shaped.glyph_ids,
                advances: shaped.advances,
                width: shaped.width,
                ascent: metrics.ascent,
                descent: metrics.descent,
                line_gap: metrics.line_gap,
                color: Color::BLACK,
                bold: style.bold,
                italic: style.italic,
                underline: None,
                strike: false,
                dstrike: false,
                highlight: None,
                baseline_offset: 0.0,
                hyperlink_url: None,
                field_kind: None,
                note: None,
            }));
        }

        let params = LineBreakParams {
            available_width: content_width,
            line_spacing: LineSpacing::Single,
            jc: Some(match paragraph.align {
                Align::Start => oxml_layout::Align::Start,
                Align::Center => oxml_layout::Align::Center,
                Align::End => oxml_layout::Align::End,
                Align::Justify => oxml_layout::Align::Justify,
            }),
            // LO/ODF 관례가 기본값: 한글은 어절 단위 줄바꿈.
            hangul_word_wrap: paragraph.hangul_word_wrap.unwrap_or(true),
            ..Default::default()
        };
        let lines = oxml_layout::break_into_lines(&items, &params, fm)?;

        cursor_y += paragraph.space_before_pt;
        for line in &lines {
            if cursor_y + line.height > content_bottom && !elements.is_empty() {
                flush_page(&mut elements, &mut pages, page_number);
                page_number += 1;
                cursor_y = page.margin_top_pt;
            }

            // 폰트 자연 행간: lineGap은 행 위에 앉는다 (LO 실측 관례).
            let baseline_y = cursor_y + line.line_gap + line.ascent;
            let extra = (content_width - line.indent_left - line.width).max(0.0);
            let mut x = page.margin_left_pt
                + line.indent_left
                + match paragraph.align {
                    Align::Center => extra / 2.0,
                    Align::End => extra,
                    Align::Start | Align::Justify => 0.0,
                };
            for item in &line.items {
                match item {
                    LineItem::Text(seg) | LineItem::Marker(seg) => {
                        if !seg.glyph_ids.is_empty() {
                            elements.push(PositionedElement::Text(oxml_layout::GlyphRun {
                                origin: Point { x, y: baseline_y },
                                font_id: seg.font_id,
                                font_size: seg.font_size,
                                glyph_ids: seg.glyph_ids.clone(),
                                advances: seg.advances.clone(),
                                text: seg.text.clone(),
                                source: None,
                                color: seg.color,
                                bold: seg.bold,
                                italic: seg.italic,
                                field_kind: None,
                                note: None,
                            }));
                        }
                        x += seg.width;
                    }
                    item => x += item.width(),
                }
            }
            cursor_y += line.height;
        }
        cursor_y += paragraph.space_after_pt;
    }

    flush_page(&mut elements, &mut pages, page_number);
    Ok(LayoutResult::new(
        pages,
        fm.all_font_data(),
        None,
        Vec::new(),
    ))
}
