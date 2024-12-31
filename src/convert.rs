use std::path::{Path, PathBuf};
use std::process::Command;

trait PngConvert: Send + Sync + std::fmt::Debug {
    fn supported(&self) -> bool;
    fn convert(&self, svgfile: &Path, pngfile: &Path) -> Result<PathBuf, String>;
}

#[derive(Debug)]
struct ImageMagick {}

impl PngConvert for ImageMagick {
    fn supported(&self) -> bool {
        Command::new("magick").arg("-version").output().is_ok()
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

#[derive(Debug)]
struct Inkscape {}

impl PngConvert for Inkscape {
    fn supported(&self) -> bool {
        Command::new("inkscape").arg("--version").output().is_ok()
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

#[derive(Debug)]
struct FallbackConverter {}

impl PngConvert for FallbackConverter {
    fn supported(&self) -> bool {
        true
    }

    fn convert(&self, _: &Path, _: &Path) -> Result<PathBuf, String> {
        Err("No supported SVG to PNG converter found".to_owned())
    }
}

pub struct PngConverter {
    converter: Box<dyn PngConvert>,
}

impl PngConverter {
    pub fn new() -> Self {
        let converter: Box<dyn PngConvert> = if (ImageMagick {}).supported() {
            Box::new(ImageMagick {}) as Box<dyn PngConvert>
        } else if (Inkscape {}).supported() {
            Box::new(Inkscape {}) as Box<dyn PngConvert>
        } else {
            Box::new(FallbackConverter {}) as Box<dyn PngConvert>
        };
        PngConverter { converter }
    }

    pub fn to_png(&self, svgfile: &Path) -> Result<PathBuf, String> {
        let mut pngfile = svgfile.to_path_buf();
        if !pngfile.set_extension("png") {
            return Err("Failed to derive SVG filename".to_owned());
        }
        self.converter.convert(svgfile, &pngfile)
    }
}
