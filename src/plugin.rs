use crate::convert::PngConverter;
use crate::transform::{svgdx_handler, TransformResult};

use serde_json::Value;

use std::cell::OnceCell;
use std::env;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::Builder;

/// How generated SVG should be embedded in the output doc
#[derive(Clone, Copy, Debug)]
pub enum EmbedType {
    // inline SVG within the doc
    Inline,
    // image link to temp SVG file
    SvgFile,
    // image link to temp PNG converted from SVG
    PngFile,
}

impl From<String> for EmbedType {
    fn from(s: String) -> Self {
        match s.as_str() {
            "markdown" | "html" | "epub" => EmbedType::Inline,
            "docx" | "pptx" => EmbedType::PngFile,
            _ => EmbedType::SvgFile,
        }
    }
}

/// Represents a block of content to be inserted into the document
enum BlockType {
    // raw HTML to be inserted as-is
    RawHtml(String),
    // link containing path to image file
    Image(PathBuf),
}

pub trait PandocPlugin {
    /// Called on each AST object; should return content of object if selected
    fn select_object(&self, object: &serde_json::Map<String, Value>) -> Option<String>;

    /// Process the selected object, replacing it with the appropriate block
    fn process_object(&self, object: &mut serde_json::Map<String, Value>, content: String);

    /// Process code blocks in the AST, replacing svgdx fenced code blocks with
    /// either raw HTML (e.g. inline SVG) or image blocks as appropriate.
    fn process_doc(&self, value: &mut Value) {
        if let Some(array) = value.as_array_mut() {
            for item in array.iter_mut() {
                self.process_doc(item);
            }
        } else if let Some(object) = value.as_object_mut() {
            if let Some(content) = self.select_object(object) {
                self.process_object(object, content);
            }
            for value in object.values_mut() {
                self.process_doc(value);
            }
        }
    }
}

pub struct SvgdxPlugin {
    embed: EmbedType,
    tmpdir: Option<String>,
    converter: OnceCell<PngConverter>,
}

impl SvgdxPlugin {
    pub fn new(embed: EmbedType, tmpdir: Option<String>) -> Self {
        SvgdxPlugin {
            embed,
            tmpdir,
            converter: OnceCell::new(),
        }
    }

    fn generate_block(&self, content: String) -> BlockType {
        let svgdx_output = &svgdx_handler(&content);
        match svgdx_output {
            TransformResult::Error(e) => BlockType::RawHtml(e.to_string()),
            TransformResult::Svg(s) => {
                if matches!(self.embed, EmbedType::Inline) {
                    BlockType::RawHtml(s.to_string())
                } else {
                    // write to temporary image file and create image link to it
                    let mut tmpfile = Builder::new()
                        .prefix("tmp-svgdx-")
                        .suffix(".svg")
                        // must persist - pandoc will need it beyond our lifetime
                        .disable_cleanup(true)
                        .tempfile_in(
                            self.tmpdir
                                .as_ref()
                                .map(Path::new)
                                .unwrap_or(&env::temp_dir()),
                        )
                        .expect("Could not create temporary file");
                    tmpfile.write_all(s.as_bytes()).unwrap();
                    let imgfile = PathBuf::from(tmpfile.path());
                    drop(tmpfile); // close file to flush
                    match self.embed {
                        EmbedType::SvgFile => BlockType::Image(imgfile),
                        EmbedType::PngFile => match self
                            .converter
                            .get_or_init(PngConverter::new)
                            .to_png(&imgfile)
                        {
                            Ok(pngfile) => BlockType::Image(pngfile),
                            // usability: better to fail hard than risk missing images
                            // through e.g. inserting error messages in generated doc
                            Err(e) => panic!("PNG conversion failed: {e}"),
                        },
                        _ => unreachable!(),
                    }
                }
            }
        }
    }
}

impl PandocPlugin for SvgdxPlugin {
    /// Extract content of `svgdx`-fenced code blocks
    fn select_object(&self, object: &serde_json::Map<String, Value>) -> Option<String> {
        if object.get("t") == Some(&Value::String("CodeBlock".to_string())) {
            if let Some(Value::Array(inner)) = object.get("c") {
                if let [Value::Array(meta), Value::String(content)] = &inner[..] {
                    if let [_, Value::Array(classes), _] = &meta[..] {
                        if classes.first() == Some(&Value::String("svgdx".to_string())) {
                            return Some(content.clone());
                        }
                    }
                }
            }
        }
        None
    }

    /// Process code blocks in the AST, replacing svgdx fenced code blocks with
    /// either raw HTML (e.g. inline SVG) or image blocks as appropriate.
    fn process_object(&self, object: &mut serde_json::Map<String, Value>, content: String) {
        match self.generate_block(content) {
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
}
