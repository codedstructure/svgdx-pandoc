use std::env::{self, args};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

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
                        if let Some("svgdx") = classes.get(0).and_then(|v| v.as_str()) {
                            return Some(content.as_str().unwrap().to_string());
                        }
                    }
                }
            }
        }
    }
    None
}

#[derive(Clone, Copy, Debug)]
enum EmbedType {
    Inline,
    SvgFile,
    PngFile,
}

impl From<String> for EmbedType {
    fn from(s: String) -> Self {
        match s.as_str() {
            "html" | "epub" => EmbedType::Inline,
            "docx" | "pptx" => EmbedType::PngFile,
            _ => EmbedType::SvgFile,
        }
    }
}

enum TransformResult {
    Svg(String),
    Error(String),
}

fn svgdx_handler(s: &str) -> TransformResult {
    match svgdx::transform_str_default(s.to_string()) {
        Ok(s) => TransformResult::Svg(s),
        Err(e) => TransformResult::Error(format!(
            r#"<div style="color: red; border: 5px double red; padding: 1em;">{}</div>"#,
            e.to_string().replace('\n', "<br/>")
        )),
    }
}

trait PngConvert: Send + Sync {
    fn supported(&self) -> bool;
    fn convert(&self, svgfile: &Path, pngfile: &Path) -> Result<PathBuf, String>;
}

struct ImageMagick {}

impl PngConvert for ImageMagick {
    fn supported(&self) -> bool {
        lazy_static::lazy_static! {
            static ref SUPPORTED: bool = Command::new("magick").arg("-version").output().is_ok();
        }
        *SUPPORTED
    }

    fn convert(&self, svgfile: &Path, pngfile: &Path) -> Result<PathBuf, String> {
        let mut cmd = Command::new("magick");
        cmd.args([
            "-density",
            "300",
            svgfile.to_str().expect("Invalid SVG path"),
            pngfile.to_str().expect("Invalid PNG path"),
        ]);
        cmd.output()
            .map(|_| pngfile.to_path_buf())
            .map_err(|e| e.to_string())
    }
}

struct Inkscape {}

impl PngConvert for Inkscape {
    fn supported(&self) -> bool {
        lazy_static::lazy_static! {
            static ref SUPPORTED: bool = Command::new("inkscape").arg("--version").output().is_ok();
        }
        *SUPPORTED
    }

    fn convert(&self, svgfile: &Path, pngfile: &Path) -> Result<PathBuf, String> {
        let mut cmd = Command::new("inkscape");
        cmd.args([
            "--export-type=png",
            "--export-dpi=300",
            "--export-filename",
            pngfile.to_str().expect("Invalid PNG path"),
            svgfile.to_str().expect("Invalid SVG path"),
        ]);
        cmd.output()
            .map(|_| pngfile.to_path_buf())
            .map_err(|e| e.to_string())
    }
}

struct FallbackConverter {}

impl PngConvert for FallbackConverter {
    fn supported(&self) -> bool {
        true
    }

    fn convert(&self, _: &Path, _: &Path) -> Result<PathBuf, String> {
        Err("No supported SVG to PNG converter found".to_owned())
    }
}

fn png_convert(svgfile: &Path) -> Result<PathBuf, String> {
    let svgfile = svgfile.to_path_buf();
    let mut pngfile = svgfile.clone();
    if !pngfile.set_extension("png") {
        return Err("Failed to derive SVG filename".to_owned());
    }
    lazy_static::lazy_static! {
        static ref CONVERTER: Box<dyn PngConvert> = if (ImageMagick{}).supported() {
            Box::new(ImageMagick{}) as Box<dyn PngConvert>
        } else if (Inkscape{}).supported() {
            Box::new(Inkscape{}) as Box<dyn PngConvert>
        } else {
            Box::new(FallbackConverter{}) as Box<dyn PngConvert>
        };
    }
    match CONVERTER.convert(&svgfile, &pngfile) {
        Ok(_) => Ok(pngfile),
        Err(e) => Err(format!("Failed to convert SVG to PNG: {}", e)),
    }
}

enum BlockType {
    RawHtml(String),
    Image(PathBuf),
}

fn error_format(s: &str) -> String {
    format!(
        r#"<div style="color: red; border: 5px double red; padding: 1em;">{}</div>"#,
        s.replace('\n', "<br/>")
    )
}

fn generate_block(content: String, embed: EmbedType, tmpdir: &Option<String>) -> BlockType {
    let svgdx_output = &svgdx_handler(&content);
    match svgdx_output {
        TransformResult::Error(e) => BlockType::RawHtml(error_format(e)),
        TransformResult::Svg(s) => {
            if matches!(embed, EmbedType::Inline) {
                BlockType::RawHtml(blank_line_remover(s))
            } else {
                // write to temporary image file and create image link to it
                let mut tmpfile = Builder::new()
                    .prefix("tmp-svgdx-")
                    .suffix(".svg")
                    // must persist - pandoc will need it beyond our lifetime
                    .keep(true)
                    .tempfile_in(tmpdir.as_ref().map(Path::new).unwrap_or(&env::temp_dir()))
                    .expect("Could not create temporary file");
                tmpfile.write_all(s.as_bytes()).unwrap();
                let imgfile = PathBuf::from(tmpfile.path());
                drop(tmpfile); // close file to flush
                match embed {
                    EmbedType::SvgFile => BlockType::Image(imgfile),
                    EmbedType::PngFile => match png_convert(&imgfile) {
                        Ok(pngfile) => BlockType::Image(pngfile),
                        Err(e) => BlockType::RawHtml(error_format(&e.to_string())),
                    },
                    _ => unreachable!(),
                }
            }
        }
    }
}

fn process_codeblocks(value: &mut Value, embed: EmbedType, tmpdir: &Option<String>) {
    if let Some(array) = value.as_array_mut() {
        for item in array.iter_mut() {
            process_codeblocks(item, embed, tmpdir);
        }
    } else if let Some(object) = value.as_object_mut() {
        if let Some(content) = is_svgdx_block(object) {
            match generate_block(content, embed, tmpdir) {
                BlockType::RawHtml(s) => {
                    *object = serde_json::json!({
                        "t": "RawBlock",
                        "c": ["html", s] //blank_line_remover(&s)]
                    })
                    .as_object()
                    .unwrap()
                    .clone();
                }
                BlockType::Image(imgfile) => {
                    *object = serde_json::json!({
                        "t": "Para",
                        "c": [{
                            "t": "Image",
                            "c": [
                                ["", [], []], [], [imgfile, ""]
                            ]
                        }]
                    })
                    .as_object()
                    .unwrap()
                    .clone();
                }
            }
        }
        for value in object.values_mut() {
            process_codeblocks(value, embed, tmpdir);
        }
    }
}

fn main() {
    let embed = args().nth(1).unwrap_or_default().into();
    // TODO: some 'force PNG' option
    let tmpdir = env::var("SVGDX_PANDOC_TMPDIR").ok();

    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut handle = stdin.lock();
    let mut output = stdout.lock();

    let mut input: Value = from_reader(&mut handle).expect("Failed to read JSON from stdin");
    process_codeblocks(&mut input, embed, &tmpdir);
    to_writer(&mut output, &input).expect("Failed to write JSON to stdout");
}
