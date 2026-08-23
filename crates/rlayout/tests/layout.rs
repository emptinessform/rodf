//! rlayout v0 — 문단 플로우 엔진 단위 테스트.
//!
//! 검증 기준은 오늘까지 오라클로 실측한 LibreOffice 관례다:
//! 행 높이 = hhea asc+desc+lineGap, gap은 행 위 배치, 한글은 어절 단위 줄바꿈.

use oxml_layout::{FontManager, PositionedElement};
use rlayout::{Align, Block, Document, PageGeometry, Paragraph, Run, TextStyle};

const A4: PageGeometry = PageGeometry {
    width_pt: 595.3,
    height_pt: 841.9,
    margin_top_pt: 56.7,
    margin_right_pt: 56.7,
    margin_bottom_pt: 56.7,
    margin_left_pt: 56.7,
};

fn para(text: &str, size: f64) -> Block {
    Block::Paragraph(Paragraph {
        runs: vec![Run {
            text: text.to_string(),
            style: TextStyle {
                font_family: Some("맑은 고딕".to_string()),
                font_size_pt: size,
                bold: false,
                italic: false,
            },
        }],
        align: Align::Start,
        space_before_pt: 0.0,
        space_after_pt: 0.0,
        hangul_word_wrap: None,
    })
}

fn text_baselines(result: &oxml_layout::LayoutResult) -> Vec<f64> {
    let mut ys: Vec<f64> = result.pages[0]
        .elements
        .iter()
        .filter_map(|e| match e {
            PositionedElement::Text(run) => Some(run.origin.y),
            _ => None,
        })
        .collect();
    ys.sort_by(|a, b| a.partial_cmp(b).unwrap());
    ys.dedup_by(|a, b| (*a - *b).abs() < 0.01);
    ys
}

/// 맑은 고딕의 기대 메트릭 (동일 시스템 폰트에서 독립 산출).
fn malgun_metrics(size: f64) -> (f64, f64) {
    let mut fm = FontManager::new();
    let id = fm.resolve_font(Some("맑은 고딕"), false, false).unwrap();
    let m = fm.metrics(id, size).unwrap();
    (m.ascent + m.descent + m.line_gap, m.line_gap + m.ascent)
}

#[test]
fn single_paragraph_page_geometry_and_baseline() {
    let doc = Document {
        page: A4,
        blocks: vec![para("안녕하세요 Hello", 12.0)],
    };
    let result = rlayout::layout(&doc).expect("layout");
    assert_eq!(result.pages.len(), 1);
    let page = &result.pages[0];
    assert!((page.width - 595.3).abs() < 0.01);
    assert!((page.height - 841.9).abs() < 0.01);

    let baselines = text_baselines(&result);
    assert_eq!(baselines.len(), 1, "one line expected: {baselines:?}");
    // 첫 베이스라인 = margin_top + lineGap + ascent (gap은 행 위)
    let (_lh, gap_plus_asc) = malgun_metrics(12.0);
    assert!(
        (baselines[0] - (56.7 + gap_plus_asc)).abs() < 0.05,
        "baseline {} vs expected {}",
        baselines[0],
        56.7 + gap_plus_asc
    );
}

#[test]
fn baselines_step_by_natural_line_height() {
    let doc = Document {
        page: A4,
        blocks: vec![
            para("첫째 문단", 12.0),
            para("둘째 문단", 12.0),
            para("셋째 문단", 12.0),
        ],
    };
    let result = rlayout::layout(&doc).expect("layout");
    let baselines = text_baselines(&result);
    assert_eq!(baselines.len(), 3, "{baselines:?}");
    let (lh, _) = malgun_metrics(12.0);
    for pair in baselines.windows(2) {
        assert!(
            ((pair[1] - pair[0]) - lh).abs() < 0.05,
            "baseline step {} vs natural line height {}",
            pair[1] - pair[0],
            lh
        );
    }
}

#[test]
fn korean_wraps_by_word_by_default() {
    // "가나다라마바사 "×20, 12pt, A4: 어절 단위면 행 콘텐츠 폭 ~436pt(5어절),
    // 음절 단위면 ~478pt까지 채움 (2026-08-23 LO 실측 기반 판별값 460pt).
    let doc = Document {
        page: A4,
        blocks: vec![para(&"가나다라마바사 ".repeat(20).trim_end(), 12.0)],
    };
    let result = rlayout::layout(&doc).expect("layout");
    // 같은 베이스라인(y)의 런 폭을 합산해 행 폭을 구한다.
    let mut by_line: Vec<(f64, f64)> = Vec::new();
    for e in &result.pages[0].elements {
        if let PositionedElement::Text(run) = e {
            let w: f64 = run.advances.iter().sum();
            match by_line.iter_mut().find(|(y, _)| (*y - run.origin.y).abs() < 0.01) {
                Some((_, total)) => *total += w,
                None => by_line.push((run.origin.y, w)),
            }
        }
    }
    let line_widths: Vec<f64> = by_line.iter().map(|(_, w)| *w).collect();
    assert_eq!(line_widths.len(), 4, "{line_widths:?}");
    for w in &line_widths {
        assert!(
            *w < 460.0,
            "line width {w} exceeds word-wrap bound (syllable filling?)"
        );
    }
}

#[test]
fn long_document_paginates() {
    let doc = Document {
        page: A4,
        blocks: (0..60).map(|i| para(&format!("문단 {i}"), 24.0)).collect(),
    };
    let result = rlayout::layout(&doc).expect("layout");
    assert!(result.pages.len() >= 2, "pages: {}", result.pages.len());
    // 모든 페이지의 요소는 하단 여백을 침범하지 않는다
    for page in &result.pages {
        for e in &page.elements {
            if let PositionedElement::Text(run) = e {
                assert!(run.origin.y < 841.9 - 56.7 + 0.01);
            }
        }
    }
}
