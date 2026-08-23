//! 스타일 모델과 해석(flatten) — automatic style + 부모 체인 + default-style.

use std::collections::HashMap;

/// 탭 스톱 정렬 종류 (style:type).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabStopAlign {
    Left,
    Center,
    Right,
}

/// 명시적 탭 스톱 하나 (style:tab-stop).
#[derive(Debug, Clone, PartialEq)]
pub struct TabStop {
    pub pos_pt: f64,
    pub align: TabStopAlign,
}

/// 문단 정렬 (fo:text-align).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Align {
    Start,
    Center,
    End,
    Justify,
}

/// style:text-properties에서 읽은 원시 속성 (미해석, Option = 미지정).
#[derive(Debug, Clone, Default)]
pub struct RawTextProps {
    pub font_name: Option<String>,
    pub font_family: Option<String>,
    pub font_size_pt: Option<f64>,
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    // ODF는 서양(fo:*)과 동아시아(style:*-asian) 속성을 분리한다.
    pub font_name_asian: Option<String>,
    pub font_size_asian_pt: Option<f64>,
    pub bold_asian: Option<bool>,
    pub italic_asian: Option<bool>,
    /// 문단 속성이지만 같은 상속 체인을 타므로 함께 나른다.
    pub align: Option<Align>,
    /// 명시적 탭 스톱 목록 — ODF에서 tab-stops는 통째로 대체된다.
    pub tab_stops: Option<Vec<TabStop>>,
    /// 문단 위/아래 여백 (fo:margin-top / fo:margin-bottom, pt).
    pub margin_top_pt: Option<f64>,
    pub margin_bottom_pt: Option<f64>,
    /// 문자 배경색 (fo:background-color 원시 값, "transparent" 포함).
    pub background_color: Option<String>,
}

impl RawTextProps {
    /// self 위에 other의 지정된 속성을 덮어쓴다 (other가 우선).
    pub(crate) fn overridden_by(&self, other: &RawTextProps) -> RawTextProps {
        RawTextProps {
            font_name: other.font_name.clone().or_else(|| self.font_name.clone()),
            font_family: other
                .font_family
                .clone()
                .or_else(|| self.font_family.clone()),
            font_size_pt: other.font_size_pt.or(self.font_size_pt),
            bold: other.bold.or(self.bold),
            italic: other.italic.or(self.italic),
            font_name_asian: other
                .font_name_asian
                .clone()
                .or_else(|| self.font_name_asian.clone()),
            font_size_asian_pt: other.font_size_asian_pt.or(self.font_size_asian_pt),
            bold_asian: other.bold_asian.or(self.bold_asian),
            italic_asian: other.italic_asian.or(self.italic_asian),
            align: other.align.or(self.align),
            tab_stops: other.tab_stops.clone().or_else(|| self.tab_stops.clone()),
            margin_top_pt: other.margin_top_pt.or(self.margin_top_pt),
            margin_bottom_pt: other.margin_bottom_pt.or(self.margin_bottom_pt),
            background_color: other
                .background_color
                .clone()
                .or_else(|| self.background_color.clone()),
        }
    }
}

/// LO 내장 Heading 기본값 — 스타일 미정의 text:h에 적용된다.
/// 기준: Heading 베이스 = Liberation Sans 14pt, 위 0.42cm/아래 0.21cm,
/// 레벨 배율 H1 130% (space.odt 오라클 실측). H2+ 배율은 LO 소스
/// (DocumentStylePoolManager) 기반 — 실측은 H1만 완료.
fn builtin_heading_props(level: u8) -> RawTextProps {
    const SIZE_PCT: [f64; 10] = [1.30, 1.15, 1.01, 0.95, 0.85, 0.75, 0.75, 0.75, 0.75, 0.75];
    let pct = SIZE_PCT[usize::from(level.clamp(1, 10)) - 1];
    let size = 14.0 * pct;
    RawTextProps {
        font_family: Some("Liberation Sans".to_string()),
        font_size_pt: Some(size),
        bold: Some(true),
        font_size_asian_pt: Some(size),
        bold_asian: Some(true),
        margin_top_pt: Some(0.42 * 72.0 / 2.54),
        margin_bottom_pt: Some(0.21 * 72.0 / 2.54),
        ..RawTextProps::default()
    }
}

/// style:style 하나 (automatic 또는 named).
#[derive(Debug, Clone, Default)]
pub struct RawStyle {
    pub parent: Option<String>,
    pub props: RawTextProps,
}

/// styles.xml에서 읽은 스타일 시트.
#[derive(Debug, Default)]
pub struct StyleSheet {
    /// office:styles의 named paragraph 스타일 (이름 → 스타일).
    pub named: HashMap<String, RawStyle>,
    /// office:styles의 named character(text) 스타일 (이름 → 스타일).
    pub named_text: HashMap<String, RawStyle>,
    /// style:default-style family=paragraph의 속성.
    pub default_props: RawTextProps,
    /// font-face-decls: style:font-name → svg:font-family.
    pub font_faces: HashMap<String, String>,
    /// style:page-layout 이름 → 페이지 기하.
    pub page_layouts: HashMap<String, PageGeometry>,
    /// 첫 style:master-page가 참조하는 page-layout-name.
    pub master_page_layout: Option<String>,
    /// default-style의 기본 탭 간격 (style:tab-stop-distance, pt).
    pub tab_stop_distance_pt: Option<f64>,
}

/// master-page가 참조하는 page-layout의 페이지 기하 (pt 단위).
/// 여백이 미지정이면 0.0 (ODF 기본 상속은 M1 범위 밖).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PageGeometry {
    pub width_pt: f64,
    pub height_pt: f64,
    pub margin_top_pt: f64,
    pub margin_right_pt: f64,
    pub margin_bottom_pt: f64,
    pub margin_left_pt: f64,
}

/// 해석 완료된 텍스트 스타일. 서양(western)과 동아시아(asian) 속성을
/// 분리해 유지한다 — 렌더러가 문자 체계별로 적용한다.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ResolvedTextStyle {
    pub font_family: Option<String>,
    pub font_size_pt: Option<f64>,
    pub bold: bool,
    pub italic: bool,
    pub font_family_asian: Option<String>,
    pub font_size_asian_pt: Option<f64>,
    pub bold_asian: bool,
    pub italic_asian: bool,
    /// 문단 정렬 (미지정 = None, 렌더러 기본은 Start).
    pub align: Option<Align>,
    /// 명시적 탭 스톱 (없으면 기본 간격의 암시 스톱만).
    pub tab_stops: Vec<TabStop>,
    /// 문단 위/아래 여백 (pt, 미지정 = 0).
    pub margin_top_pt: f64,
    pub margin_bottom_pt: f64,
    /// 문자 배경색 (해석 완료, transparent/미지정 = None).
    pub background_rgb: Option<(u8, u8, u8)>,
}

/// 문단 style-name → 해석된 스타일. automatic(content.xml) 우선,
/// named(styles.xml) 폴백, 부모 체인은 named를 따라 올라간다.
pub struct StyleResolver {
    sheet: StyleSheet,
    automatic: HashMap<String, RawStyle>,
    /// content.xml의 font-face-decls (styles.xml 것과 병합).
    font_faces: HashMap<String, String>,
}

impl StyleResolver {
    pub fn new(sheet: StyleSheet, content_auto: ContentStyles) -> Self {
        let mut font_faces = sheet.font_faces.clone();
        font_faces.extend(content_auto.font_faces);
        StyleResolver {
            sheet,
            automatic: content_auto.styles,
            font_faces,
        }
    }

    /// 스타일 체인(automatic leaf → named 부모들)을 flatten한 원시 속성.
    /// `text_family`가 true면 named 조회를 character 스타일 맵에서 한다.
    fn chain_props(&self, style_name: &str, text_family: bool) -> RawTextProps {
        let named = if text_family {
            &self.sheet.named_text
        } else {
            &self.sheet.named
        };
        let mut chain: Vec<&RawStyle> = Vec::new();
        let mut cursor = Some(style_name.to_string());
        let mut first = true;
        while let Some(name) = cursor.take() {
            let style = if first {
                self.automatic.get(&name).or_else(|| named.get(&name))
            } else {
                named.get(&name)
            };
            first = false;
            let Some(style) = style else { break };
            cursor = style.parent.clone();
            chain.push(style);
            if chain.len() > 32 {
                break; // 순환 방어
            }
        }
        let mut props = RawTextProps::default();
        for style in chain.iter().rev() {
            props = props.overridden_by(&style.props);
        }
        props
    }

    /// 문단 스타일의 원시 속성. 순서: default-style ← 내장 Heading
    /// (text:h일 때) ← named/automatic 체인 — 문서 정의가 항상 이긴다.
    fn paragraph_props(&self, style_name: Option<&str>, heading_level: Option<u8>) -> RawTextProps {
        let mut props = self.sheet.default_props.clone();
        if let Some(level) = heading_level {
            props = props.overridden_by(&builtin_heading_props(level));
        }
        if let Some(name) = style_name {
            props = props.overridden_by(&self.chain_props(name, false));
        }
        props
    }

    pub fn resolve(
        &self,
        style_name: Option<&str>,
        heading_level: Option<u8>,
    ) -> ResolvedTextStyle {
        self.finish(self.paragraph_props(style_name, heading_level))
    }

    /// 문단 스타일 위에 스팬 스타일들(바깥→안쪽)을 덮어 해석한다.
    pub fn resolve_span(
        &self,
        paragraph_style: Option<&str>,
        heading_level: Option<u8>,
        span_styles: &[String],
    ) -> ResolvedTextStyle {
        let mut props = self.paragraph_props(paragraph_style, heading_level);
        for name in span_styles {
            props = props.overridden_by(&self.chain_props(name, true));
        }
        self.finish(props)
    }

    fn finish(&self, props: RawTextProps) -> ResolvedTextStyle {
        let resolve_family = |font_name: &Option<String>| {
            font_name.as_ref().map(|name| {
                self.font_faces
                    .get(name)
                    .map(|fam| strip_quotes(fam))
                    .unwrap_or_else(|| name.clone())
            })
        };
        let font_family = props
            .font_family
            .clone()
            .or_else(|| resolve_family(&props.font_name));
        let font_family_asian = resolve_family(&props.font_name_asian);

        ResolvedTextStyle {
            font_family,
            font_size_pt: props.font_size_pt,
            bold: props.bold.unwrap_or(false),
            italic: props.italic.unwrap_or(false),
            font_family_asian,
            font_size_asian_pt: props.font_size_asian_pt,
            bold_asian: props.bold_asian.unwrap_or(false),
            italic_asian: props.italic_asian.unwrap_or(false),
            align: props.align,
            tab_stops: props.tab_stops.unwrap_or_default(),
            margin_top_pt: props.margin_top_pt.unwrap_or(0.0),
            margin_bottom_pt: props.margin_bottom_pt.unwrap_or(0.0),
            background_rgb: props
                .background_color
                .as_deref()
                .and_then(parse_hex_color),
        }
    }
}

/// content.xml에서 읽은 automatic style 묶음.
#[derive(Debug, Default)]
pub struct ContentStyles {
    pub styles: HashMap<String, RawStyle>,
    pub font_faces: HashMap<String, String>,
}

fn strip_quotes(s: &str) -> String {
    s.trim_matches(|c| c == '\'' || c == '"').to_string()
}

/// "#rrggbb"를 RGB로. "transparent" 등 비색상 값은 None.
fn parse_hex_color(v: &str) -> Option<(u8, u8, u8)> {
    let hex = v.trim().strip_prefix('#')?;
    if hex.len() != 6 {
        return None;
    }
    Some((
        u8::from_str_radix(&hex[0..2], 16).ok()?,
        u8::from_str_radix(&hex[2..4], 16).ok()?,
        u8::from_str_radix(&hex[4..6], 16).ok()?,
    ))
}
