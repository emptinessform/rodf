//! rodf-render 어댑터(경로 α) 첫 테스트 — hello.odt를 rdocx 엔진으로
//! 레이아웃해 PNG/PDF를 얻고, 매핑 손실이 없음을 확인한다.

use rodf_core::Document;

fn fixture() -> Document {
    Document::open(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../rodf-core/tests/fixtures/hello.odt"
    ))
    .expect("hello.odt should open")
}

#[test]
fn renders_hello_to_single_page() {
    let rendered = rodf_render::render(&fixture()).expect("render should succeed");
    assert_eq!(rendered.page_count(), 1);
}

#[test]
fn page_png_has_png_signature() {
    let rendered = rodf_render::render(&fixture()).expect("render should succeed");
    let png = rendered.page_png(0, 144.0).expect("page 0 should render");
    assert!(png.starts_with(&[0x89, b'P', b'N', b'G']));
}

#[test]
fn pdf_has_pdf_signature() {
    let rendered = rodf_render::render(&fixture()).expect("render should succeed");
    assert!(rendered.pdf().starts_with(b"%PDF"));
}

#[test]
fn hello_maps_without_losses() {
    let rendered = rodf_render::render(&fixture()).expect("render should succeed");
    assert!(
        rendered.losses().is_empty(),
        "unexpected mapping losses: {:?}",
        rendered.losses()
    );
}

/// PNG IHDR에서 (width, height)를 읽는다.
fn png_size(png: &[u8]) -> (u32, u32) {
    let w = u32::from_be_bytes(png[16..20].try_into().unwrap());
    let h = u32::from_be_bytes(png[20..24].try_into().unwrap());
    (w, h)
}

#[test]
fn page_size_follows_odf_master_page_a4() {
    let rendered = rodf_render::render(&fixture()).expect("render should succeed");
    let png = rendered.page_png(0, 144.0).expect("page 0 should render");
    let (w, h) = png_size(&png);
    // A4 @144dpi: 595.3pt*2 = ~1191, 841.9pt*2 = ~1684 (±2px)
    assert!((1189..=1193).contains(&w), "width {w} is not A4 at 144dpi");
    assert!((1682..=1686).contains(&h), "height {h} is not A4 at 144dpi");
}

mod script_split {
    use rodf_render::{split_script_runs, Script};

    #[test]
    fn splits_mixed_korean_latin_heading() {
        assert_eq!(
            split_script_runs("안녕하세요 Hello — rodf"),
            vec![
                (Script::Asian, "안녕하세요 ".to_string()),
                (Script::Western, "Hello — rodf".to_string()),
            ]
        );
    }

    #[test]
    fn pure_latin_is_single_western_run() {
        assert_eq!(
            split_script_runs("Hello"),
            vec![(Script::Western, "Hello".to_string())]
        );
    }

    #[test]
    fn leading_whitespace_joins_first_strong_run() {
        assert_eq!(
            split_script_runs("  안녕"),
            vec![(Script::Asian, "  안녕".to_string())]
        );
    }

    #[test]
    fn empty_text_yields_no_runs() {
        assert!(split_script_runs("").is_empty());
    }
}

#[test]
fn paragraphs_get_odf_spacing_defaults_not_word_defaults() {
    use rdocx_oxml::{BodyContent, Twips};
    // ODF/LO 기본: 문단 간격 0, 단일 행간 — Word Normal(1.08행간, after-spacing)을
    // 그대로 두면 문단 수직 위치가 오라클과 어긋난다.
    let (input, _losses) = rodf_render::to_layout_input(&fixture());
    for content in &input.document.body.content {
        let BodyContent::Paragraph(p) = content else { continue };
        let ppr = p.properties.as_ref().expect("paragraph properties set");
        assert_eq!(ppr.space_before, Some(Twips(0)));
        assert_eq!(ppr.space_after, Some(Twips(0)));
        assert_eq!(ppr.line_spacing, Some(Twips(240)));
        assert_eq!(ppr.line_rule.as_deref(), Some("auto"));
    }
}
