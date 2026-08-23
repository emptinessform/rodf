//! M1 첫 테스트 — LibreOffice가 만든 hello.odt를 파싱해
//! 문단 텍스트와 automatic style 해석(flatten) 결과를 검증한다.

use rodf_core::Document;

fn fixture() -> Document {
    Document::open(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/hello.odt"
    ))
    .expect("hello.odt should open")
}

#[test]
fn opens_hello_odt_with_two_paragraphs() {
    let doc = fixture();
    assert_eq!(doc.paragraphs().len(), 2);
}

#[test]
fn first_paragraph_resolves_automatic_style() {
    let doc = fixture();
    let p = &doc.paragraphs()[0];
    assert!(p.text().contains("안녕하세요 Hello"));
    let style = p.style();
    assert_eq!(style.font_size_pt, Some(24.0));
    assert!(style.bold);
    assert_eq!(style.font_family.as_deref(), Some("맑은 고딕"));
}

#[test]
fn second_paragraph_falls_back_to_named_style() {
    let doc = fixture();
    let p = &doc.paragraphs()[1];
    assert!(p.text().contains("본문 크기 문단"));
    let style = p.style();
    assert!(!style.bold);
    assert_ne!(style.font_size_pt, Some(24.0));
}

#[test]
fn resolves_a4_page_geometry_from_master_page() {
    let doc = fixture();
    let geometry = doc.page_geometry().expect("hello.odt has a master page");
    // 21.001cm x 29.7cm, 여백 2cm (pt 환산, 오차 0.5pt 허용)
    assert!((geometry.width_pt - 595.3).abs() < 0.5, "width {}", geometry.width_pt);
    assert!((geometry.height_pt - 841.9).abs() < 0.5, "height {}", geometry.height_pt);
    assert!((geometry.margin_top_pt - 56.7).abs() < 0.5);
    assert!((geometry.margin_left_pt - 56.7).abs() < 0.5);
    assert!((geometry.margin_right_pt - 56.7).abs() < 0.5);
    assert!((geometry.margin_bottom_pt - 56.7).abs() < 0.5);
}

#[test]
fn heading_keeps_asian_properties_separate_from_western() {
    let doc = fixture();
    let style = doc.paragraphs()[0].style();
    // P1은 fo:font-size=24pt(서양)만 지정 — asian은 default-style의 10pt 유지
    assert_eq!(style.font_size_pt, Some(24.0));
    assert!(style.bold);
    assert_eq!(style.font_size_asian_pt, Some(10.0));
    assert!(!style.bold_asian);
}
