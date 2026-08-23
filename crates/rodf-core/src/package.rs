//! ODF ZIP 패키지 계층 — mimetype 검증과 주요 파트 로드.

use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::OdfError;

const ODT_MIME: &str = "application/vnd.oasis.opendocument.text";

/// 열린 ODT 패키지의 주요 XML 파트.
pub struct OdtPackage {
    pub content_xml: String,
    pub styles_xml: String,
}

impl OdtPackage {
    pub fn open(path: &Path) -> Result<Self, OdfError> {
        let mut zip = zip::ZipArchive::new(File::open(path)?)?;

        let mimetype = read_entry(&mut zip, "mimetype")?;
        if mimetype.trim() != ODT_MIME {
            return Err(OdfError::WrongMimeType(mimetype.trim().to_string()));
        }

        let content_xml = read_entry(&mut zip, "content.xml")?;
        let styles_xml = read_entry(&mut zip, "styles.xml")?;
        Ok(OdtPackage {
            content_xml,
            styles_xml,
        })
    }
}

fn read_entry(
    zip: &mut zip::ZipArchive<File>,
    name: &'static str,
) -> Result<String, OdfError> {
    let mut entry = zip
        .by_name(name)
        .map_err(|_| OdfError::MissingEntry(name))?;
    let mut buf = String::new();
    entry.read_to_string(&mut buf)?;
    Ok(buf)
}
