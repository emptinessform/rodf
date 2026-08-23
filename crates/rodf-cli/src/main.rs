//! rodf — ODT 렌더링 CLI.
//!
//! 사용법: rodf render <input.odt> <output.{pdf,png}>

use std::path::Path;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [cmd, input, output] if cmd == "render" => render(Path::new(input), Path::new(output)),
        _ => {
            eprintln!("usage: rodf render <input.odt> <output.pdf|output.png>");
            ExitCode::from(2)
        }
    }
}

fn render(input: &Path, output: &Path) -> ExitCode {
    let doc = match rodf_core::Document::open(input) {
        Ok(doc) => doc,
        Err(e) => {
            eprintln!("rodf: cannot open {}: {e}", input.display());
            return ExitCode::FAILURE;
        }
    };

    for note in doc.coverage_notes() {
        eprintln!("rodf: coverage: {} x{}", note.element, note.count);
    }

    let rendered = match rodf_render::render(&doc) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("rodf: render failed: {e}");
            return ExitCode::FAILURE;
        }
    };
    for loss in rendered.losses() {
        eprintln!("rodf: mapping loss [{}]: {}", loss.what, loss.detail);
    }

    let extension = output
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase);
    let bytes = match extension.as_deref() {
        Some("pdf") => rendered.pdf(),
        Some("png") => match rendered.page_png(0, 144.0) {
            Some(png) => png,
            None => {
                eprintln!("rodf: document has no page 0");
                return ExitCode::FAILURE;
            }
        },
        _ => {
            eprintln!("rodf: output must end in .pdf or .png");
            return ExitCode::from(2);
        }
    };

    if let Err(e) = std::fs::write(output, bytes) {
        eprintln!("rodf: cannot write {}: {e}", output.display());
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
