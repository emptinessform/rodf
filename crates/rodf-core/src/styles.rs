//! 스타일 모델과 해석(flatten) — automatic style + 부모 체인 + default-style.

use std::collections::HashMap;

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
}

impl RawTextProps {
    /// self 위에 other의 지정된 속성을 덮어쓴다 (other가 우선).
    fn overridden_by(&self, other: &RawTextProps) -> RawTextProps {
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
        }
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
    /// style:default-style family=paragraph의 속성.
    pub default_props: RawTextProps,
    /// font-face-decls: style:font-name → svg:font-family.
    pub font_faces: HashMap<String, String>,
    /// style:page-layout 이름 → 페이지 기하.
    pub page_layouts: HashMap<String, PageGeometry>,
    /// 첫 style:master-page가 참조하는 page-layout-name.
    pub master_page_layout: Option<String>,
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

    pub fn resolve(&self, style_name: Option<&str>) -> ResolvedTextStyle {
        let mut chain: Vec<&RawStyle> = Vec::new();
        let mut cursor = style_name.map(str::to_string);
        // leaf → root 순으로 체인 수집 (automatic은 leaf에서만 나올 수 있음).
        let mut first = true;
        while let Some(name) = cursor.take() {
            let style = if first {
                self.automatic
                    .get(&name)
                    .or_else(|| self.sheet.named.get(&name))
            } else {
                self.sheet.named.get(&name)
            };
            first = false;
            let Some(style) = style else { break };
            cursor = style.parent.clone();
            chain.push(style);
            if chain.len() > 32 {
                break; // 순환 방어
            }
        }

        // default → root → ... → leaf 순서로 덮어쓰기.
        let mut props = self.sheet.default_props.clone();
        for style in chain.iter().rev() {
            props = props.overridden_by(&style.props);
        }

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
