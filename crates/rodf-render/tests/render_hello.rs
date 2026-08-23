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
        assert_eq!(ppr.line_rule.as_deref(), Some("font-natural"));
    }
}

mod line_gap {
    use rodf_core::Document;

    fn gulim() -> Document {
        Document::open(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/gulim.odt"
        ))
        .expect("gulim.odt should open")
    }

    /// PNG 회색조 근사로 잉크 밴드(y0,y1)를 추출한다.
    fn ink_bands(png: &[u8]) -> Vec<(u32, u32)> {
        let decoder = png::Decoder::new(std::io::Cursor::new(png));
        let mut reader = decoder.read_info().unwrap();
        let mut buf = vec![0u8; reader.output_buffer_size().unwrap()];
        let info = reader.next_frame(&mut buf).unwrap();
        let (w, h) = (info.width as usize, info.height as usize);
        let bpp = info.buffer_size() / (w * h);
        let mut bands = Vec::new();
        let mut in_band = false;
        let mut start = 0u32;
        for y in 0..h {
            let row_has_ink = (0..w).any(|x| buf[(y * w + x) * bpp] < 245);
            if row_has_ink && !in_band {
                start = y as u32;
                in_band = true;
            } else if !row_has_ink && in_band {
                bands.push((start, y as u32));
                in_band = false;
            }
        }
        bands
    }

    /// 굴림(hhea lineGap=152/1024)의 행 높이는 gap 포함 1.1484em이어야 한다.
    /// LO 실측: 24pt에서 주기 55px, 첫 잉크 y=125 (gap이 행 위에 배치됨).
    #[test]
    fn gulim_line_period_includes_line_gap() {
        let rendered = rodf_render::render(&gulim()).expect("render should succeed");
        let png = rendered.page_png(0, 144.0).expect("page 0");
        let bands = ink_bands(&png);
        assert_eq!(bands.len(), 4, "four paragraphs expected: {bands:?}");
        let periods: Vec<i64> = bands
            .windows(2)
            .map(|w| w[1].0 as i64 - w[0].0 as i64)
            .collect();
        for p in &periods {
            assert!(
                (54..=56).contains(p),
                "period {p} should be ~55px (1.1484em incl lineGap); all={periods:?}"
            );
        }
    }

    /// LO는 lineGap을 행 위에 배치한다 — 첫 잉크가 그만큼 내려와야 한다.
    #[test]
    fn gulim_line_gap_sits_above_the_line() {
        let rendered = rodf_render::render(&gulim()).expect("render should succeed");
        let png = rendered.page_png(0, 144.0).expect("page 0");
        let bands = ink_bands(&png);
        let first_ink = bands[0].0;
        assert!(
            (123..=128).contains(&first_ink),
            "first ink y {first_ink} should be ~125 (margin + gap + ascent - glyph extent)"
        );
    }
}

mod synthetic_italic {
    use rodf_core::Document;

    /// 밴드 내 행별 잉크 x-중심의 선형 기울기 (최소제곱).
    /// 합성 기울임이 적용되면 글자가 오른쪽으로 누워 기울기가 음의 방향으로
    /// tan(20°)≈0.364만큼 이동한다 (DirectWrite oblique 관례, LO 실측 0.358).
    fn ink_slope(png: &[u8], y0: usize, y1: usize) -> f64 {
        let decoder = png::Decoder::new(std::io::Cursor::new(png));
        let mut reader = decoder.read_info().unwrap();
        let mut buf = vec![0u8; reader.output_buffer_size().unwrap()];
        let info = reader.next_frame(&mut buf).unwrap();
        let (w, h) = (info.width as usize, info.height as usize);
        let bpp = info.buffer_size() / (w * h);
        let mut pts: Vec<(f64, f64)> = Vec::new();
        for y in y0..y1.min(h) {
            let xs: Vec<f64> = (0..w)
                .filter(|&x| buf[(y * w + x) * bpp] < 245)
                .map(|x| x as f64)
                .collect();
            if !xs.is_empty() {
                pts.push((y as f64, xs.iter().sum::<f64>() / xs.len() as f64));
            }
        }
        let n = pts.len() as f64;
        let (sy, sx): (f64, f64) = pts.iter().fold((0.0, 0.0), |a, p| (a.0 + p.0, a.1 + p.1));
        let (my, mx) = (sy / n, sx / n);
        let num: f64 = pts.iter().map(|p| (p.0 - my) * (p.1 - mx)).sum();
        let den: f64 = pts.iter().map(|p| (p.0 - my) * (p.0 - my)).sum();
        num / den
    }

    /// 이탤릭 페이스가 없는 맑은 고딕의 이탤릭 문단은 합성 스큐로 렌더돼야 한다.
    /// 직립 실측 기울기 -0.68(글리프 분포 편향), 20° 스큐 적용 시 ≈ -1.05.
    #[test]
    fn italic_paragraph_is_sheared_when_face_has_no_italic() {
        let doc = Document::open(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../corpus/bold-italic.odt"
        ))
        .expect("bold-italic.odt should open");
        let rendered = rodf_render::render(&doc).expect("render");
        let png = rendered.page_png(0, 144.0).expect("page 0");
        // 이탤릭 문단 밴드 (144dpi, 고정 픽스처): y 157..184
        let slope = ink_slope(&png, 157, 184);
        assert!(
            (-1.25..=-0.90).contains(&slope),
            "italic line slope {slope:.3} should be ~-1.05 (upright is ~-0.68)"
        );
    }
}
