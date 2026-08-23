//! content.xml / styles.xml 파서 — quick-xml 풀 파서.
//!
//! 요소/속성은 접두사를 뗀 로컬 이름으로 비교한다 (LO는 표준 접두사를
//! 쓰지만 다른 생산자를 배제하지 않기 위해).

use std::collections::HashMap;

use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;

use crate::styles::{Align, ContentStyles, PageGeometry, RawStyle, RawTextProps, StyleSheet};
use crate::OdfError;

/// content.xml 파싱 결과.
#[derive(Debug, Default)]
pub struct Content {
    pub automatic_styles: ContentStyles,
    pub paragraphs: Vec<RawParagraph>,
    /// 미지원 구조 요소 집계 (요소 로컬명 → 등장 횟수). 해당 서브트리는
    /// 파싱에서 제외된다 — 조용히 사라지는 대신 커버리지 미달로 보고한다.
    pub unsupported: HashMap<String, usize>,
}

/// 아직 렌더 경로가 없는 구조 요소 — 서브트리째 건너뛰고 집계한다.
/// (text:section은 투명 컨테이너라 여기 없다 — 내용은 정상 파싱된다.)
fn unsupported_kind(local: &[u8]) -> Option<&'static str> {
    match local {
        b"table" => Some("table"),
        b"frame" => Some("frame"),
        b"list" => Some("list"),
        b"image" => Some("image"),
        b"note" => Some("note"),                 // 각주/미주 — 본문에 섞이면 안 됨
        b"bibliography" => Some("bibliography"), // 참고문헌 인덱스
        b"table-of-content" => Some("table-of-content"),
        b"alphabetical-index" => Some("alphabetical-index"),
        b"custom-shape" => Some("custom-shape"),
        b"object" => Some("object"),
        b"control" => Some("control"),           // 양식 컨트롤
        b"forms" => Some("forms"),
        _ => None,
    }
}

fn parse_align(value: &str) -> Option<Align> {
    match value {
        "start" | "left" => Some(Align::Start),
        "end" | "right" => Some(Align::End),
        "center" => Some(Align::Center),
        "justify" => Some(Align::Justify),
        _ => None,
    }
}

fn read_para_align(e: &BytesStart) -> Option<Align> {
    attr_local(e, "text-align").and_then(|v| parse_align(&v))
}

#[derive(Debug)]
pub struct RawParagraph {
    pub style_name: Option<String>,
    pub text: String,
}

fn local_name(qname: &[u8]) -> &[u8] {
    match qname.iter().position(|&b| b == b':') {
        Some(i) => &qname[i + 1..],
        None => qname,
    }
}

fn attr_local(e: &BytesStart, name: &str) -> Option<String> {
    for attr in e.attributes().flatten() {
        if local_name(attr.key.as_ref()) == name.as_bytes() {
            return attr
                .normalized_value(quick_xml::XmlVersion::default())
                .ok()
                .map(|v| v.into_owned());
        }
    }
    None
}

fn strip_quotes(s: &str) -> String {
    s.trim_matches(|c| c == '\'' || c == '"').to_string()
}

fn parse_font_size(value: &str) -> Option<f64> {
    value.strip_suffix("pt")?.trim().parse().ok()
}

/// ODF 길이("21.001cm", "20mm", "1in", "595.3pt")를 pt로 환산한다.
fn parse_length_pt(value: &str) -> Option<f64> {
    let value = value.trim();
    let (number, factor) = if let Some(n) = value.strip_suffix("cm") {
        (n, 72.0 / 2.54)
    } else if let Some(n) = value.strip_suffix("mm") {
        (n, 72.0 / 25.4)
    } else if let Some(n) = value.strip_suffix("in") {
        (n, 72.0)
    } else if let Some(n) = value.strip_suffix("pt") {
        (n, 1.0)
    } else {
        return None;
    };
    number.trim().parse::<f64>().ok().map(|v| v * factor)
}

fn read_page_layout_props(e: &BytesStart) -> PageGeometry {
    let length = |name: &str| attr_local(e, name).and_then(|v| parse_length_pt(&v));
    PageGeometry {
        width_pt: length("page-width").unwrap_or(0.0),
        height_pt: length("page-height").unwrap_or(0.0),
        margin_top_pt: length("margin-top").unwrap_or(0.0),
        margin_right_pt: length("margin-right").unwrap_or(0.0),
        margin_bottom_pt: length("margin-bottom").unwrap_or(0.0),
        margin_left_pt: length("margin-left").unwrap_or(0.0),
    }
}

fn parse_font_weight(value: &str) -> Option<bool> {
    match value {
        "bold" => Some(true),
        "normal" => Some(false),
        other => other.parse::<u32>().ok().map(|w| w >= 600),
    }
}

fn read_text_props(e: &BytesStart) -> RawTextProps {
    RawTextProps {
        font_name: attr_local(e, "font-name"),
        font_family: attr_local(e, "font-family").map(|f| strip_quotes(&f)),
        font_size_pt: attr_local(e, "font-size").and_then(|v| parse_font_size(&v)),
        bold: attr_local(e, "font-weight").and_then(|v| parse_font_weight(&v)),
        italic: attr_local(e, "font-style").map(|v| v == "italic"),
        font_name_asian: attr_local(e, "font-name-asian"),
        font_size_asian_pt: attr_local(e, "font-size-asian").and_then(|v| parse_font_size(&v)),
        bold_asian: attr_local(e, "font-weight-asian").and_then(|v| parse_font_weight(&v)),
        italic_asian: attr_local(e, "font-style-asian").map(|v| v == "italic"),
        align: None, // paragraph-properties에서 별도로 채워진다
    }
}

fn read_font_face(e: &BytesStart, faces: &mut HashMap<String, String>) {
    if let (Some(name), Some(family)) = (attr_local(e, "name"), attr_local(e, "font-family")) {
        faces.insert(name, strip_quotes(&family));
    }
}

/// styles.xml을 파싱해 named 스타일/default-style/font-face를 모은다.
pub fn parse_styles_xml(xml: &str) -> Result<StyleSheet, OdfError> {
    let mut reader = Reader::from_str(xml);
    let mut sheet = StyleSheet::default();

    enum Scope {
        None,
        Named(String, RawStyle),
        Default(RawTextProps),
    }
    let mut scope = Scope::None;
    let mut in_office_styles = false;
    let mut current_page_layout: Option<String> = None;

    loop {
        match reader.read_event()? {
            Event::Start(e) => match local_name(e.name().as_ref()) {
                b"styles" => in_office_styles = true,
                b"page-layout" => current_page_layout = attr_local(&e, "name"),
                b"page-layout-properties" => {
                    if let Some(name) = &current_page_layout {
                        sheet
                            .page_layouts
                            .insert(name.clone(), read_page_layout_props(&e));
                    }
                }
                b"master-page" => {
                    if sheet.master_page_layout.is_none() {
                        sheet.master_page_layout = attr_local(&e, "page-layout-name");
                    }
                }
                b"style" if in_office_styles => {
                    if attr_local(&e, "family").as_deref() == Some("paragraph") {
                        scope = Scope::Named(
                            attr_local(&e, "name").unwrap_or_default(),
                            RawStyle {
                                parent: attr_local(&e, "parent-style-name"),
                                props: RawTextProps::default(),
                            },
                        );
                    }
                }
                b"default-style" => {
                    if attr_local(&e, "family").as_deref() == Some("paragraph") {
                        scope = Scope::Default(RawTextProps::default());
                    }
                }
                b"paragraph-properties" => {
                    if let Some(align) = read_para_align(&e) {
                        match &mut scope {
                            Scope::Named(_, style) => style.props.align = Some(align),
                            Scope::Default(p) => p.align = Some(align),
                            Scope::None => {}
                        }
                    }
                }
                _ => {}
            },
            Event::Empty(e) => match local_name(e.name().as_ref()) {
                b"font-face" => read_font_face(&e, &mut sheet.font_faces),
                b"page-layout-properties" => {
                    if let Some(name) = &current_page_layout {
                        sheet
                            .page_layouts
                            .insert(name.clone(), read_page_layout_props(&e));
                    }
                }
                b"master-page" => {
                    if sheet.master_page_layout.is_none() {
                        sheet.master_page_layout = attr_local(&e, "page-layout-name");
                    }
                }
                b"text-properties" => {
                    let props = read_text_props(&e);
                    match &mut scope {
                        Scope::Named(_, style) => {
                            let align = style.props.align;
                            style.props = props;
                            style.props.align = style.props.align.or(align);
                        }
                        Scope::Default(p) => {
                            let align = p.align;
                            *p = props;
                            p.align = p.align.or(align);
                        }
                        Scope::None => {}
                    }
                }
                b"paragraph-properties" => {
                    if let Some(align) = read_para_align(&e) {
                        match &mut scope {
                            Scope::Named(_, style) => style.props.align = Some(align),
                            Scope::Default(p) => p.align = Some(align),
                            Scope::None => {}
                        }
                    }
                }
                // 자식 없는 style:style — 속성 없는 스타일로 등록.
                b"style" if in_office_styles => {
                    if attr_local(&e, "family").as_deref() == Some("paragraph") {
                        sheet.named.insert(
                            attr_local(&e, "name").unwrap_or_default(),
                            RawStyle {
                                parent: attr_local(&e, "parent-style-name"),
                                props: RawTextProps::default(),
                            },
                        );
                    }
                }
                _ => {}
            },
            Event::End(e) => match local_name(e.name().as_ref()) {
                b"styles" => in_office_styles = false,
                b"page-layout" => current_page_layout = None,
                b"style" => {
                    if let Scope::Named(name, style) = std::mem::replace(&mut scope, Scope::None) {
                        sheet.named.insert(name, style);
                    }
                }
                b"default-style" => {
                    if let Scope::Default(props) = std::mem::replace(&mut scope, Scope::None) {
                        sheet.default_props = props;
                    }
                }
                _ => {}
            },
            Event::Eof => break,
            _ => {}
        }
    }
    Ok(sheet)
}

/// content.xml을 파싱해 automatic style과 문단 시퀀스를 모은다.
pub fn parse_content_xml(xml: &str) -> Result<Content, OdfError> {
    let mut reader = Reader::from_str(xml);
    let mut content = Content::default();

    let mut in_automatic = false;
    let mut current_style: Option<(String, RawStyle)> = None;

    // text:p 내부 텍스트 수집 상태. span 등 중첩 요소 깊이를 추적한다.
    let mut para: Option<RawParagraph> = None;
    let mut para_depth = 0usize;
    // 미지원 서브트리 스킵 깊이 (0 = 스킵 아님).
    let mut skip_depth = 0usize;

    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                if skip_depth > 0 {
                    skip_depth += 1;
                    continue;
                }
                if let Some(kind) = unsupported_kind(local_name(e.name().as_ref())) {
                    *content.unsupported.entry(kind.to_string()).or_default() += 1;
                    skip_depth = 1;
                    continue;
                }
                if para.is_some() {
                    para_depth += 1;
                } else {
                    match local_name(e.name().as_ref()) {
                        b"automatic-styles" => in_automatic = true,
                        b"paragraph-properties" => {
                            if let Some(align) = read_para_align(&e) {
                                if let Some((_, style)) = &mut current_style {
                                    style.props.align = Some(align);
                                }
                            }
                        }
                        b"style" if in_automatic => {
                            let family = attr_local(&e, "family");
                            if family.as_deref() == Some("paragraph")
                                || family.as_deref() == Some("text")
                            {
                                current_style = Some((
                                    attr_local(&e, "name").unwrap_or_default(),
                                    RawStyle {
                                        parent: attr_local(&e, "parent-style-name"),
                                        props: RawTextProps::default(),
                                    },
                                ));
                            }
                        }
                        // text:h(제목)는 text:p와 같은 문단 흐름이다.
                        b"p" | b"h" => {
                            para = Some(RawParagraph {
                                style_name: attr_local(&e, "style-name"),
                                text: String::new(),
                            });
                            para_depth = 1;
                        }
                        _ => {}
                    }
                }
            }
            Event::Empty(e) => match local_name(e.name().as_ref()) {
                _ if skip_depth > 0 => {}
                kind_name if unsupported_kind(kind_name).is_some() => {
                    let kind = unsupported_kind(kind_name).unwrap();
                    *content.unsupported.entry(kind.to_string()).or_default() += 1;
                }
                // 빈 문단(<text:p/>) — LO에서 한 행을 차지하므로 보존한다.
                b"p" | b"h" if para.is_none() => {
                    content.paragraphs.push(RawParagraph {
                        style_name: attr_local(&e, "style-name"),
                        text: String::new(),
                    });
                }
                b"font-face" => read_font_face(&e, &mut content.automatic_styles.font_faces),
                b"text-properties" => {
                    if let Some((_, style)) = &mut current_style {
                        let align = style.props.align;
                        style.props = read_text_props(&e);
                        style.props.align = style.props.align.or(align);
                    }
                }
                b"paragraph-properties" => {
                    if let Some(align) = read_para_align(&e) {
                        if let Some((_, style)) = &mut current_style {
                            style.props.align = Some(align);
                        }
                    }
                }
                b"s" => {
                    if let Some(p) = &mut para {
                        let count: usize = attr_local(&e, "c")
                            .and_then(|v| v.parse().ok())
                            .unwrap_or(1);
                        p.text.extend(std::iter::repeat_n(' ', count));
                    }
                }
                b"tab" => {
                    if let Some(p) = &mut para {
                        p.text.push('\t');
                    }
                    // 탭 스톱은 아직 미해석 — 커버리지로 보고한다.
                    *content.unsupported.entry("tab".to_string()).or_default() += 1;
                }
                b"line-break" => {
                    if let Some(p) = &mut para {
                        p.text.push('\n');
                    }
                }
                _ => {}
            },
            Event::Text(t) => {
                if skip_depth > 0 {
                    continue;
                }
                if let Some(p) = &mut para {
                    p.text.push_str(&t.decode()?);
                }
            }
            Event::End(e) => {
                if skip_depth > 0 {
                    skip_depth -= 1;
                    continue;
                }
                if para.is_some() {
                    para_depth -= 1;
                    if para_depth == 0 {
                        content.paragraphs.push(para.take().unwrap());
                    }
                } else {
                    match local_name(e.name().as_ref()) {
                        b"automatic-styles" => in_automatic = false,
                        b"style" => {
                            if let Some((name, style)) = current_style.take() {
                                content.automatic_styles.styles.insert(name, style);
                            }
                        }
                        _ => {}
                    }
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }
    Ok(content)
}
