# rodf

**Lightweight, high-fidelity ODF (OpenDocument) rendering in pure Rust.** ODT first.

rodf is the ODF member of a family of Rust document-format projects:
[rdocx](https://github.com/tensorbee/rdocx) (DOCX library + layout engine),
[rdoc](https://github.com/emptinessform/rdoc) (browser DOCX viewer/editor), and
[rhwp](https://github.com/edwardkim/rhwp) (HWP/HWPX full stack).

The gap rodf targets: LibreOffice-grade ODT rendering without shipping LibreOffice.
ZetaOffice ports the whole suite to WASM (hundreds of MB); pure-Rust ODF crates focus
on document *generation* without layout. rodf aims at the middle — a small library
that parses ODT and renders it faithfully to PDF/PNG, with **Korean typography as a
first-class requirement**.

## Status: early (M1 in progress)

`rodf render in.odt out.pdf` works for text documents today:

- ODT package + `content.xml` / `styles.xml` parsing
- Automatic-style / named-style / default-style chain resolution, with Western
  (`fo:*`) and East Asian (`style:*-asian`) properties kept separate — mixed
  Korean/Latin text renders at its correct per-script size and weight
- Master-page geometry (page size, margins)
- Rendering through the [rdocx](https://github.com/tensorbee/rdocx) layout engine
  (adapter approach), PDF and PNG output

LibreOffice (left) vs rodf (right), same `hello.odt`:

![LibreOffice vs rodf side-by-side](docs/side-by-side.png)

Every change is judged against a **LibreOffice oracle** — `tools/oracle.py` renders
the same document through headless LibreOffice and computes a content-cropped SSIM.
Current score on the hello fixture: **0.49** against a 0.95 target; the remaining gap
is line-height model differences (Word-style vs LibreOffice font-metric-proportional
spacing), which drives the roadmap below.

## Crates

| Crate | Role |
|---|---|
| `rodf-core` | ODF package + document model + style resolution (zip + quick-xml only) |
| `rodf-render` | ODF → layout-engine adapter, PDF/PNG output, mapping-loss tracking |
| `rodf-cli` | `rodf render in.odt out.{pdf,png}` |

## Roadmap

- **M1** — single-paragraph fidelity: parse → render → oracle SSIM ≥ 0.95
- **M1.5** — oracle corpus (50–100 public ODT files) + pinned-LibreOffice Docker CI
- **M2** — format-neutral layout engine work (the adapter's mapping-loss list decides
  whether rodf keeps adapting or gets its own flow engine)
- **M2.5** — public fidelity dashboard ("Are we ODF yet?")
- **M3+** — tables, images, headers/footers, SVG backend, ODS/ODP, WASM/npm

## Development

```sh
cargo test --workspace          # 18 tests, all written test-first
cargo run -p rodf-cli -- render crates/rodf-core/tests/fixtures/hello.odt out.pdf
python tools/oracle.py crates/rodf-core/tests/fixtures/hello.odt   # needs LibreOffice
```

## License

MIT OR Apache-2.0
