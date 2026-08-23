//! content.xml / styles.xml 파서 — quick-xml 풀 파서.
//!
//! 요소/속성은 접두사를 뗀 로컬 이름으로 비교한다 (LO는 표준 접두사를
//! 쓰지만 다른 생산자를 배제하지 않기 위해).

use std::collections::HashMap;

use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;

use crate::styles::{Align, ContentStyles, PageGeometry, RawStyle, RawTextProps, StyleSheet, TabStop, TabStopAlign};
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

/// paragraph-properties의 문단 속성(정렬/상하 여백)을 props에 반영한다.
fn apply_para_props(e: &BytesStart, props: &mut RawTextProps) {
    if let Some(align) = read_para_align(e) {
        props.align = Some(align);
    }
    if let Some(v) = attr_local(e, "margin-top").and_then(|v| parse_length_pt(&v)) {
        props.margin_top_pt = Some(v);
    }
    if let Some(v) = attr_local(e, "margin-bottom").and_then(|v| parse_length_pt(&v)) {
        props.margin_bottom_pt = Some(v);
    }
}

fn read_tab_stop(e: &BytesStart) -> Option<TabStop> {
    let pos_pt = attr_local(e, "position").and_then(|v| parse_length_pt(&v))?;
    let align = match attr_local(e, "type").as_deref() {
        Some("center") => TabStopAlign::Center,
        Some("right") => TabStopAlign::Right,
        _ => TabStopAlign::Left, // left / char(근사) / 미지정
    };
    Some(TabStop { pos_pt, align })
}

#[derive(Debug)]
pub struct RawParagraph {
    pub style_name: Option<String>,
    /// text:h의 아웃라인 레벨 (text:p는 None).
    pub outline_level: Option<u8>,
    pub segments: Vec<RawSegment>,
}

/// 문단 내 한 스팬 구간 — 적용된 스팬 스타일 스택(바깥→안쪽)과 텍스트.
#[derive(Debug)]
pub struct RawSegment {
    pub span_styles: Vec<String>,
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
        align: None,     // paragraph-properties에서 별도로 채워진다
        tab_stops: None, // tab-stops 컨테이너에서 별도로 채워진다
        margin_top_pt: None,
        margin_bottom_pt: None,
        background_color: attr_local(e, "background-color"),
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
        /// (이름, 스타일, character(text) 패밀리 여부)
        Named(String, RawStyle, bool),
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
                    let family = attr_local(&e, "family");
                    if matches!(family.as_deref(), Some("paragraph") | Some("text")) {
                        scope = Scope::Named(
                            attr_local(&e, "name").unwrap_or_default(),
                            RawStyle {
                                parent: attr_local(&e, "parent-style-name"),
                                props: RawTextProps::default(),
                            },
                            family.as_deref() == Some("text"),
                        );
                    }
                }
                b"default-style" => {
                    if attr_local(&e, "family").as_deref() == Some("paragraph") {
                        scope = Scope::Default(RawTextProps::default());
                    }
                }
                b"paragraph-properties" => {
                    match &mut scope {
                        Scope::Named(_, style, _) => apply_para_props(&e, &mut style.props),
                        Scope::Default(p) => apply_para_props(&e, p),
                        Scope::None => {}
                    }
                    if matches!(scope, Scope::Default(_)) {
                        if let Some(d) = attr_local(&e, "tab-stop-distance")
                            .and_then(|v| parse_length_pt(&v))
                        {
                            if d > 0.0 {
                                sheet.tab_stop_distance_pt = Some(d);
                            }
                        }
                    }
                }
                // tab-stops 컨테이너 등장 = 상속 스톱 전체 대체 (ODF 규칙)
                b"tab-stops" => match &mut scope {
                    Scope::Named(_, style, _) => style.props.tab_stops = Some(Vec::new()),
                    Scope::Default(p) => p.tab_stops = Some(Vec::new()),
                    Scope::None => {}
                },
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
                b"tab-stop" => {
                    if let Some(stop) = read_tab_stop(&e) {
                        let slot = match &mut scope {
                            Scope::Named(_, style, _) => Some(&mut style.props.tab_stops),
                            Scope::Default(p) => Some(&mut p.tab_stops),
                            Scope::None => None,
                        };
                        if let Some(slot) = slot {
                            slot.get_or_insert_with(Vec::new).push(stop);
                        }
                    }
                }
                b"text-properties" => {
                    // 병합으로 문단 속성(align/tabs/margins)은 그대로 보존된다.
                    let props = read_text_props(&e);
                    match &mut scope {
                        Scope::Named(_, style, _) => {
                            style.props = style.props.overridden_by(&props);
                        }
                        Scope::Default(p) => *p = p.overridden_by(&props),
                        Scope::None => {}
                    }
                }
                b"paragraph-properties" => {
                    match &mut scope {
                        Scope::Named(_, style, _) => apply_para_props(&e, &mut style.props),
                        Scope::Default(p) => apply_para_props(&e, p),
                        Scope::None => {}
                    }
                    // LO는 default-style의 paragraph-properties를 자식 없이
                    // 자기닫힘으로 쓰므로 여기(Empty)서도 읽어야 한다.
                    if matches!(scope, Scope::Default(_)) {
                        if let Some(d) = attr_local(&e, "tab-stop-distance")
                            .and_then(|v| parse_length_pt(&v))
                        {
                            if d > 0.0 {
                                sheet.tab_stop_distance_pt = Some(d);
                            }
                        }
                    }
                }
                // 자식 없는 style:style — 속성 없는 스타일로 등록.
                b"style" if in_office_styles => {
                    let family = attr_local(&e, "family");
                    let entry = RawStyle {
                        parent: attr_local(&e, "parent-style-name"),
                        props: RawTextProps::default(),
                    };
                    match family.as_deref() {
                        Some("paragraph") => {
                            sheet.named.insert(attr_local(&e, "name").unwrap_or_default(), entry);
                        }
                        Some("text") => {
                            sheet
                                .named_text
                                .insert(attr_local(&e, "name").unwrap_or_default(), entry);
                        }
                        _ => {}
                    }
                }
                _ => {}
            },
            Event::End(e) => match local_name(e.name().as_ref()) {
                b"styles" => in_office_styles = false,
                b"page-layout" => current_page_layout = None,
                b"style" => {
                    if let Scope::Named(name, style, is_text) =
                        std::mem::replace(&mut scope, Scope::None)
                    {
                        if is_text {
                            sheet.named_text.insert(name, style);
                        } else {
                            sheet.named.insert(name, style);
                        }
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

    // text:p 내부 수집 상태. span 스타일 스택과 세그먼트 버퍼를 유지한다.
    let mut para: Option<RawParagraph> = None;
    let mut para_depth = 0usize;
    let mut span_stack: Vec<(usize, String)> = Vec::new();
    let mut seg_buf = String::new();
    // ODF 1.2 공백 병합 상태: 문단 시작은 true(선두 공백 제거),
    // 리터럴 공백 방출 시 true, text:s/tab/line-break 등 콘텐츠는 false로 리셋.
    let mut last_was_space = true;

    // 현재 버퍼를 세그먼트로 확정한다.
    fn flush_segment(
        para: &mut Option<RawParagraph>,
        span_stack: &[(usize, String)],
        seg_buf: &mut String,
    ) {
        if seg_buf.is_empty() {
            return;
        }
        if let Some(p) = para {
            p.segments.push(RawSegment {
                span_styles: span_stack.iter().map(|(_, n)| n.clone()).collect(),
                text: std::mem::take(seg_buf),
            });
        } else {
            seg_buf.clear();
        }
    }
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
                    if local_name(e.name().as_ref()) == b"span" {
                        flush_segment(&mut para, &span_stack, &mut seg_buf);
                        if let Some(name) = attr_local(&e, "style-name") {
                            span_stack.push((para_depth + 1, name));
                        }
                    }
                    para_depth += 1;
                } else {
                    match local_name(e.name().as_ref()) {
                        b"automatic-styles" => in_automatic = true,
                        b"paragraph-properties" => {
                            if let Some((_, style)) = &mut current_style {
                                apply_para_props(&e, &mut style.props);
                            }
                        }
                        b"tab-stops" => {
                            if let Some((_, style)) = &mut current_style {
                                style.props.tab_stops = Some(Vec::new());
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
                        tag @ (b"p" | b"h") => {
                            para = Some(RawParagraph {
                                style_name: attr_local(&e, "style-name"),
                                outline_level: (tag == b"h").then(|| {
                                    attr_local(&e, "outline-level")
                                        .and_then(|v| v.parse().ok())
                                        .unwrap_or(1)
                                }),
                                segments: Vec::new(),
                            });
                            para_depth = 1;
                            span_stack.clear();
                            seg_buf.clear();
                            last_was_space = true;
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
                tag @ (b"p" | b"h") if para.is_none() => {
                    content.paragraphs.push(RawParagraph {
                        style_name: attr_local(&e, "style-name"),
                        outline_level: (tag == b"h").then(|| {
                            attr_local(&e, "outline-level")
                                .and_then(|v| v.parse().ok())
                                .unwrap_or(1)
                        }),
                        segments: Vec::new(),
                    });
                }
                b"font-face" => read_font_face(&e, &mut content.automatic_styles.font_faces),
                b"text-properties" => {
                    if let Some((_, style)) = &mut current_style {
                        // 병합 — 문단 속성(align/tabs/margins)은 보존된다.
                        style.props = style.props.overridden_by(&read_text_props(&e));
                    }
                }
                b"paragraph-properties" => {
                    if let Some((_, style)) = &mut current_style {
                        apply_para_props(&e, &mut style.props);
                    }
                }
                b"s" => {
                    if para.is_some() {
                        let count: usize = attr_local(&e, "c")
                            .and_then(|v| v.parse().ok())
                            .unwrap_or(1);
                        seg_buf.extend(std::iter::repeat_n(' ', count));
                        last_was_space = false;
                    }
                }
                b"tab" => {
                    if para.is_some() {
                        seg_buf.push('\t');
                    }
                }
                b"tab-stop" => {
                    if let Some(stop) = read_tab_stop(&e) {
                        if let Some((_, style)) = &mut current_style {
                            style.props.tab_stops.get_or_insert_with(Vec::new).push(stop);
                        }
                    }
                }
                b"line-break" => {
                    if para.is_some() {
                        seg_buf.push('\n');
                    }
                }
                _ => {}
            },
            Event::Text(t) => {
                if skip_depth > 0 {
                    continue;
                }
                if para.is_some() {
                    // 문자 데이터의 연속 공백은 스팬 경계를 넘어 1개로 병합한다.
                    for ch in t.decode()?.chars() {
                        if matches!(ch, ' ' | '\t' | '\n' | '\r') {
                            if !last_was_space {
                                seg_buf.push(' ');
                                last_was_space = true;
                            }
                        } else {
                            seg_buf.push(ch);
                            last_was_space = false;
                        }
                    }
                }
            }
            // 엔티티/문자 참조 (&lt; &#65; ...) — quick-xml이 Text와 분리해
            // 방출하므로 여기서 문자로 복원한다. 공백 문자는 병합 대상이 아니다
            // (참조로 쓴 공백은 의도된 공백).
            Event::GeneralRef(r) => {
                if skip_depth > 0 || para.is_none() {
                    continue;
                }
                let ch = if let Ok(Some(c)) = r.resolve_char_ref() {
                    Some(c)
                } else {
                    match r.decode()?.as_ref() {
                        "lt" => Some('<'),
                        "gt" => Some('>'),
                        "amp" => Some('&'),
                        "apos" => Some('\''),
                        "quot" => Some('"'),
                        _ => None, // 미지 엔티티는 버린다 (DTD 미지원)
                    }
                };
                if let Some(ch) = ch {
                    seg_buf.push(ch);
                    last_was_space = false;
                }
            }
            Event::End(e) => {
                if skip_depth > 0 {
                    skip_depth -= 1;
                    continue;
                }
                if para.is_some() {
                    if let Some((open_depth, _)) = span_stack.last() {
                        if *open_depth == para_depth {
                            flush_segment(&mut para, &span_stack, &mut seg_buf);
                            span_stack.pop();
                        }
                    }
                    para_depth -= 1;
                    if para_depth == 0 {
                        flush_segment(&mut para, &span_stack, &mut seg_buf);
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
