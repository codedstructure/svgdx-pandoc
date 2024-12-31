//! # svgdx-pandoc
//!
//! A [pandoc](https://pandoc.org) filter to embed SVG images generated from
//! [`svgdx`](https://svgdx.net)-fenced code blocks in your documents.
//!
//! ## Usage
//!
//! ```sh
//! pandoc --filter svgdx-pandoc [input.md] -o [output.ext]
//! ```
//!
//! The filter scans for `svgdx`-fenced code blocks in the input document, and
//! depending on the output format specified, will:
//!
//! - embed the SVG directly in the document (e.g. for HTML, EPUB, Markdown)
//! - write the generated SVG to a temp file and generate a link to it
//! - convert the generated SVG to a temp PNG file, and generate a link to it
//!
//! The currently supported output formats and the corresponding behaviour are:
//!
//! - `markdown`, `html`, `epub`: embed SVG directly in the document
//! - `docx`, `pptx`: convert SVG to PNG and link to the PNG in the doc
//! - other formats (e.g. `pdf`): write SVG to a temp file and embed a link to it

mod convert;
mod plugin;
mod transform;

use std::env;
use std::io;

use plugin::{PandocPlugin, SvgdxPlugin};
use serde_json::{from_reader, to_writer, Value};

fn main() {
    let embed = env::args().nth(1).unwrap_or_default().into();
    // TODO: some 'force PNG' option
    let tmpdir = env::var("SVGDX_PANDOC_TMPDIR").ok();

    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut handle = stdin.lock();
    let mut output = stdout.lock();

    let plugin = SvgdxPlugin::new(embed, tmpdir.clone());

    let mut input: Value = from_reader(&mut handle).expect("Failed to read JSON from stdin");
    plugin.process_doc(&mut input);
    to_writer(&mut output, &input).expect("Failed to write JSON to stdout");
}
