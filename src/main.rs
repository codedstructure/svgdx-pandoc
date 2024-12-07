use std::env::{self, args};
use std::io::{self, Write};
use std::path::Path;

use serde_json::{from_reader, to_writer, Value};
use tempfile::Builder;

fn blank_line_remover(s: &str) -> String {
    // Need to avoid blank lines in the rendered SVG, as they can cause
    // markdown to resume 'normal' md processing, especially when e.g.
    // indentation can cause an implicit code block to be started.
    // See https://talk.commonmark.org/t/inline-html-breaks-when-using-indentation/3317
    // and https://spec.commonmark.org/0.31.2/#html-blocks
    s.lines()
        .filter(|line| !line.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn is_svgdx_block(object: &serde_json::Map<String, Value>) -> Option<String> {
    if let Some(tag) = object.get("t") {
        if tag == "CodeBlock" {
            if let Some(inner) = object.get("c").and_then(|v| v.as_array()) {
                if let [meta, content] = inner.as_slice() {
                    // expand array to the three components we expect
                    if let [_ident, classes, _attrs] = meta.as_array().unwrap().as_slice() {
                        if classes.get(0).unwrap().as_str() == Some("svgdx") {
                            return Some(content.as_str().unwrap().to_string());
                        }
                    }
                }
            }
        }
    }
    None
}

fn svgdx_handler(s: &str) -> String {
    svgdx::transform_str_default(s.to_string()).unwrap_or_else(|e| {
        format!(
            r#"<div style="color: red; border: 5px double red; padding: 1em;">{}</div>"#,
            e.to_string().replace('\n', "<br/>")
        )
    })
}

fn supports_inline_svg(mode: &str) -> bool {
    mode == "html" || mode == "epub"
}

fn process_codeblocks(value: &mut Value, mode: &Option<String>, tmpdir: &Option<String>) {
    if let Some(array) = value.as_array_mut() {
        for item in array.iter_mut() {
            process_codeblocks(item, mode, tmpdir);
        }
    } else if let Some(object) = value.as_object_mut() {
        if let Some(content) = is_svgdx_block(object) {
            let svg_output = blank_line_remover(&svgdx_handler(&content));
            if let Some(true) = mode.as_ref().map(|s| supports_inline_svg(s)) {
                *object = serde_json::json!({
                    "t": "RawBlock",
                    "c": ["html", svg_output]
                })
                .as_object()
                .unwrap()
                .clone();
            } else {
                // write to temporary image file and create image link to it
                let mut imgfile = Builder::new()
                    .prefix("tmp-svgdx-")
                    .suffix(".svg")
                    // must persist - pandoc will need it beyond our lifetime
                    .keep(true)
                    .tempfile_in(tmpdir.as_ref().map(Path::new).unwrap_or(&env::temp_dir()))
                    .expect("Could not create temporary file");

                imgfile.write_all(svg_output.as_bytes()).unwrap();
                *object = serde_json::json!({
                    "t": "Para",
                    "c": [{
                        "t": "Image",
                        "c": [
                            ["", [], []], [], [imgfile.path(), ""]
                        ]
                    }]
                })
                .as_object()
                .unwrap()
                .clone();
            }
        }
        for value in object.values_mut() {
            process_codeblocks(value, mode, tmpdir);
        }
    }
}

fn main() {
    let mode = args().nth(1);
    let tmpdir = env::var("SVGDX_PANDOC_TMPDIR").ok();

    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut handle = stdin.lock();
    let mut output = stdout.lock();

    let mut input: Value = from_reader(&mut handle).expect("Failed to read JSON from stdin");
    process_codeblocks(&mut input, &mode, &tmpdir);
    to_writer(&mut output, &input).expect("Failed to write JSON to stdout");
}
