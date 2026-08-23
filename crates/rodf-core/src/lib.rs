//! rodf-core — 순수 Rust ODF(OpenDocument) 패키지·문서 모델. ODT 우선.
//!
//! 초기 지원 범위(설계 문서): ODF 1.2+ 표준 ZIP 패키지의 ODT만.
//! flat ODF(.fodt)/암호화/손상 파일은 명시적 에러.

mod package;
mod parse;
mod styles;

use std::path::Path;

pub use styles::{Align, PageGeometry, ResolvedTextStyle, TabStop, TabStopAlign};

/// 파싱은 됐지만 렌더 경로가 없어 건너뛴 구조 요소의 집계.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoverageNote {
    /// 요소 로컬명 (예: "table", "frame", "list", "image").
    pub element: String,
    /// 등장 횟수.
    pub count: usize,
}

/// 문단 내 한 스팬 — 문자 스타일이 적용된 연속 텍스트.
#[derive(Debug, Clone)]
pub struct Span {
    pub text: String,
    pub style: ResolvedTextStyle,
}

/// 파싱된 문단 하나. 스팬 시퀀스와 문단 수준 스타일을 보관한다.
#[derive(Debug, Clone)]
pub struct Paragraph {
    text: String,
    spans: Vec<Span>,
    style: ResolvedTextStyle,
}

impl Paragraph {
    /// 전체 텍스트 (스팬 연결).
    pub fn text(&self) -> &str {
        &self.text
    }

    /// 문단 수준 해석 스타일 (정렬·탭 스톱 등; 텍스트 속성은 스팬 기준).
    pub fn style(&self) -> &ResolvedTextStyle {
        &self.style
    }

    /// 문자 스타일이 해석된 스팬 시퀀스. 스팬 없는 문단은 단일 스팬.
    pub fn spans(&self) -> &[Span] {
        &self.spans
    }
}

/// 열린 ODT 문서. 문단 시퀀스와 해석된 스타일을 노출한다.
#[derive(Debug)]
pub struct Document {
    paragraphs: Vec<Paragraph>,
    page_geometry: Option<PageGeometry>,
    coverage_notes: Vec<CoverageNote>,
    default_tab_interval_pt: Option<f64>,
}

#[derive(Debug, thiserror::Error)]
pub enum OdfError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("zip error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("xml error: {0}")]
    Xml(#[from] quick_xml::Error),
    #[error("xml encoding error: {0}")]
    XmlEncoding(#[from] quick_xml::encoding::EncodingError),
    #[error("not an ODF text document: mimetype is {0:?}")]
    WrongMimeType(String),
    #[error("package entry missing: {0}")]
    MissingEntry(&'static str),
}

impl Document {
    /// ODT 패키지를 열어 content.xml/styles.xml을 파싱하고
    /// automatic style 체인을 해석한다.
    pub fn open(path: impl AsRef<Path>) -> Result<Document, OdfError> {
        let pkg = package::OdtPackage::open(path.as_ref())?;
        let style_sheet = parse::parse_styles_xml(&pkg.styles_xml)?;
        let content = parse::parse_content_xml(&pkg.content_xml)?;

        let page_geometry = style_sheet
            .master_page_layout
            .as_ref()
            .and_then(|name| style_sheet.page_layouts.get(name))
            .cloned();

        let default_tab_interval_pt = style_sheet.tab_stop_distance_pt;
        let content_unsupported = content.unsupported;
        let resolver = styles::StyleResolver::new(style_sheet, content.automatic_styles);
        let paragraphs = content
            .paragraphs
            .into_iter()
            .map(|p| {
                let para_style = resolver.resolve(p.style_name.as_deref(), p.outline_level);
                let mut spans: Vec<Span> = p
                    .segments
                    .iter()
                    .map(|seg| Span {
                        style: if seg.span_styles.is_empty() {
                            para_style.clone()
                        } else {
                            resolver.resolve_span(
                                p.style_name.as_deref(),
                                p.outline_level,
                                &seg.span_styles,
                            )
                        },
                        text: seg.text.clone(),
                    })
                    .collect();
                if spans.is_empty() {
                    spans.push(Span {
                        text: String::new(),
                        style: para_style.clone(),
                    });
                }
                Paragraph {
                    text: spans.iter().map(|s| s.text.as_str()).collect(),
                    spans,
                    style: para_style,
                }
            })
            .collect();
        let mut coverage_notes: Vec<CoverageNote> = content_unsupported
            .into_iter()
            .map(|(element, count)| CoverageNote { element, count })
            .collect();
        coverage_notes.sort_by(|a, b| a.element.cmp(&b.element));
        Ok(Document {
            paragraphs,
            page_geometry,
            coverage_notes,
            default_tab_interval_pt,
        })
    }

    pub fn paragraphs(&self) -> &[Paragraph] {
        &self.paragraphs
    }

    /// 첫 master-page의 페이지 기하 (pt).
    pub fn page_geometry(&self) -> Option<&PageGeometry> {
        self.page_geometry.as_ref()
    }

    /// 렌더 경로가 없어 건너뛴 요소들 — 비어 있으면 문서 전체가 지원 범위다.
    pub fn coverage_notes(&self) -> &[CoverageNote] {
        &self.coverage_notes
    }

    /// 문서가 지정한 기본 탭 간격 (pt). 미지정이면 앱 기본(1.25cm)을 쓴다.
    pub fn default_tab_interval_pt(&self) -> Option<f64> {
        self.default_tab_interval_pt
    }
}
