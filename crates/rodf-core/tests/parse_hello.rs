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

mod coverage {
    use rodf_core::Document;

    /// LibreOffice로 저작한, 표·이미지 프레임·목록·제목(text:h)이 든 픽스처.
    fn features() -> Document {
        Document::open(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/features.odt"
        ))
        .expect("features.odt should open")
    }

    /// text:h(제목)는 text:p와 같은 문단 흐름이다 — 조용히 버려지면 안 된다.
    #[test]
    fn heading_elements_are_paragraphs() {
        let doc = features();
        assert!(
            doc.paragraphs().iter().any(|p| p.text().contains("제목입니다")),
            "text:h content missing: {:?}",
            doc.paragraphs().iter().map(|p| p.text()).collect::<Vec<_>>()
        );
    }

    /// 미지원 구조 요소(표/프레임/목록)는 조용히 사라지지 말고 집계돼야 한다.
    #[test]
    fn unsupported_elements_are_reported() {
        let doc = features();
        let notes = doc.coverage_notes();
        let kinds: Vec<&str> = notes.iter().map(|n| n.element.as_str()).collect();
        assert!(kinds.contains(&"table"), "table not reported: {kinds:?}");
        assert!(kinds.contains(&"frame"), "frame not reported: {kinds:?}");
        assert!(kinds.contains(&"list"), "list not reported: {kinds:?}");
    }

    /// 지원 범위 안의 문서(hello)는 커버리지 노트가 없어야 한다.
    #[test]
    fn supported_documents_have_no_notes() {
        let doc = Document::open(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/hello.odt"
        ))
        .unwrap();
        assert!(doc.coverage_notes().is_empty());
    }
}

/// ODT 템플릿(.ott, text-template mimetype)도 같은 문서 구조다 — 열려야 한다.
#[test]
fn opens_text_template_mimetype() {
    let doc = rodf_core::Document::open(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../corpus-wild/fdo53210.odt"
    ))
    .expect("text-template should open");
    let _ = doc.paragraphs();
}

mod paragraph_align {
    use rodf_core::{Align, Document};

    /// fo:text-align이 automatic style 체인으로 해석돼야 한다.
    #[test]
    fn center_alignment_is_parsed() {
        let doc = Document::open(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../corpus-wild/paste-first-para-direct-format.odt"
        ))
        .expect("open");
        assert_eq!(doc.paragraphs()[0].style().align, Some(Align::Center));
        assert_eq!(doc.paragraphs()[1].style().align, Some(Align::Center));
    }

    /// 정렬 미지정 문서는 None (렌더러 기본 = Start).
    #[test]
    fn unspecified_alignment_is_none() {
        let doc = Document::open(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/hello.odt"
        ))
        .unwrap();
        assert_eq!(doc.paragraphs()[0].style().align, None);
    }
}

mod coverage_wild {
    use rodf_core::Document;

    fn kinds(path: &str) -> Vec<String> {
        let doc = Document::open(path).expect("open");
        doc.coverage_notes().iter().map(|n| n.element.clone()).collect()
    }

    /// 각주(text:note)는 본문에 섞이지 말고 스킵+집계돼야 한다.
    #[test]
    fn footnotes_are_reported_not_flattened() {
        let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../corpus-wild");
        let k = kinds(&format!("{root}/ooo32780-1.odt"));
        assert!(k.contains(&"note".to_string()), "{k:?}");
    }

    /// 탭·양식·도형·인덱스류도 커버리지로 보고된다.
    #[test]
    fn tabs_forms_shapes_and_indexes_are_reported() {
        let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../corpus-wild");
        // 탭은 이제 지원 기능 — 더 이상 커버리지로 보고되지 않는다.
        assert!(!kinds(&format!("{root}/space.odt")).contains(&"tab".to_string()));
        assert!(kinds(&format!("{root}/dateFormFormats.odt")).contains(&"control".to_string()));
        assert!(kinds(&format!("{root}/Word2010AsCharShape.odt")).contains(&"custom-shape".to_string()));
        assert!(kinds(&format!("{root}/BibliographyEntryField.odt")).contains(&"bibliography".to_string()));
    }

    /// text:section은 투명 컨테이너 — 내용 문단은 살아 있어야 한다.
    #[test]
    fn sections_are_transparent() {
        let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../corpus-wild");
        let doc = Document::open(&format!("{root}/ooo32780-1.odt")).unwrap();
        assert!(doc.paragraphs().len() >= 20, "sections must not swallow paragraphs: {}", doc.paragraphs().len());
    }
}

mod spans {
    use rodf_core::Document;

    fn doc() -> Document {
        Document::open(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/spans.odt"
        ))
        .expect("spans.odt should open")
    }

    /// text:span은 문단 스타일 위에 문자 스타일을 덮은 세그먼트가 된다.
    /// 중첩 스팬은 바깥 스팬 속성을 상속한 채 안쪽 속성을 더한다.
    #[test]
    fn spans_carry_character_styles() {
        let d = doc();
        let p = &d.paragraphs()[0];
        let spans = p.spans();
        let texts: Vec<&str> = spans.iter().map(|s| s.text.as_str()).collect();
        assert_eq!(texts, vec!["일반 ", "굵게큰 ", "중첩", " 꼬리"]);

        assert!(!spans[0].style.bold);
        assert_eq!(spans[0].style.font_size_pt, Some(12.0));

        assert!(spans[1].style.bold, "T1 bold");
        assert_eq!(spans[1].style.font_size_pt, Some(20.0));
        assert!(!spans[1].style.italic);

        // 중첩: T1(bold, 20pt) 상속 + T2(italic)
        assert!(spans[2].style.bold, "nested keeps outer bold");
        assert!(spans[2].style.italic, "nested adds italic");
        assert_eq!(spans[2].style.font_size_pt, Some(20.0));

        assert!(!spans[3].style.bold, "tail returns to paragraph style");
        assert_eq!(spans[3].style.font_size_pt, Some(12.0));
    }

    /// 스팬 없는 문단은 문단 스타일의 단일 스팬이다 (기존 동작 보존).
    #[test]
    fn spanless_paragraph_is_one_span() {
        let d = Document::open(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/hello.odt"
        ))
        .unwrap();
        let p = &d.paragraphs()[1];
        assert_eq!(p.spans().len(), 1);
        assert_eq!(p.spans()[0].text, p.text());
    }
}

mod whitespace {
    use rodf_core::Document;

    /// ODF 1.2 공백 병합: 문자 데이터의 연속 공백(스페이스/탭/개행)은
    /// 스팬 경계를 넘어 1개로 병합, 문단 선두 공백 제거, text:s는
    /// 무조건 방출 + 병합 상태 리셋 (corpus-wild/space.odt 기대값 실측).
    #[test]
    fn odf_whitespace_collapsing() {
        let d = Document::open(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/whitespace.odt"
        ))
        .expect("whitespace.odt should open");
        let texts: Vec<&str> = d.paragraphs().iter().map(|p| p.text()).collect();
        assert_eq!(
            texts,
            vec![
                "a b",       // 연속 리터럴 2개 → 1개
                "leading",   // 문단 선두 공백 제거
                "a b",       // 스팬 경계 넘어 병합 ("a " + " b")
                "a    b",    // s(1) + 리터럴(리셋 후 생존) + s(2) = 4칸
                "a b",       // 스페이스+탭문자+스페이스 → 1개
                // 엔티티 참조는 문자로 복원된다 (quick-xml GeneralRef)
                "<span>a & b's \"q\" AB",
            ]
        );
    }
}

mod headings {
    use rodf_core::Document;

    /// 스타일 정의가 없는 text:h는 LO 내장 Heading 기본값을 받는다:
    /// Liberation Sans, 14pt x 레벨 배율(H1 130%), bold,
    /// 위 0.42cm / 아래 0.21cm 여백. (space.odt 오라클 실측 근거)
    #[test]
    fn builtin_heading_defaults() {
        let d = Document::open(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/heading.odt"
        ))
        .expect("heading.odt should open");
        let ps = d.paragraphs();
        assert_eq!(ps.len(), 3);

        let h1 = ps[0].style();
        assert_eq!(h1.font_family.as_deref(), Some("Liberation Sans"));
        assert!((h1.font_size_pt.unwrap() - 18.2).abs() < 0.01, "H1 = 14pt x 130%");
        assert!(h1.bold);
        assert!((h1.margin_top_pt - 11.9055).abs() < 0.01, "0.42cm above");
        assert!((h1.margin_bottom_pt - 5.9528).abs() < 0.01, "0.21cm below");

        let body = ps[1].style();
        assert!(!body.bold);
        assert_eq!(body.margin_top_pt, 0.0);

        let h2 = ps[2].style();
        assert!((h2.font_size_pt.unwrap() - 16.1).abs() < 0.01, "H2 = 14pt x 115%");
        assert!(h2.bold);
    }
}

mod background {
    use rodf_core::Document;

    /// fo:background-color는 문자 배경으로 해석된다 ("transparent"는 없음).
    #[test]
    fn span_background_color() {
        let d = Document::open(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/background.odt"
        ))
        .expect("background.odt should open");
        let spans = d.paragraphs()[0].spans();
        assert_eq!(spans.len(), 2);
        assert_eq!(spans[0].style.background_rgb, None);
        assert_eq!(spans[1].style.background_rgb, Some((0, 255, 0)));
    }
}
