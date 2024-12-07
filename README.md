# svgdx-pandoc

A [pandoc][] filter to convert [svgdx][] fenced code blocks into inline SVG images.

[pandoc]: https://pandoc.org
[svgdx]: https://github.com/codedstructure/svgdx

## Installation

For now installation requires a working Rust toolchain, e.g. installed from [rustup.rs](https://rustup.rs).

Install `svgdx-pandoc` as follows:

```
cargo install svgdx-pandoc
```

## Usage

Basic usage of the filter is as follows; note the output format is inferred from the output file extension.

```
pandoc --filter svgdx-pandoc input.md -o output.html
```

An [example Markdown file](example/example.md) with embedded `svgdx` diagrams is present in the [example/](/example/) directory.

To create various other documents from this, try the following after cloning this repository. Note this assumes both pandoc and this project are installed.

```bash
svgdx-pandoc$ cd example/
svgdx-pandoc/example$
svgdx-pandoc/example$ for FMT in html epub pdf; do pandoc --filter svgdx-pandoc example.md -o example.${FMT} ; done
```

Try opening the various generated files; note that some output formats tend to work more reliably than others,
for example `.docx` files may not preserve styles accurately. Converting images to PNG (e.g. via Inkscape)
may be appropriate in this case and may be added in a future version.

## How it works

This filter has two modes of operation, selected automatically based on the target document format
(which pandoc provides as the sole argument to a filter):

* For **html** and **epub** output, the rendered SVG is included inline within the rendered document;
  for example a generated HTML document will contain an SVG element at the appropriate location.
* For other formats (e.g. **PDF**, **odt**), raw inline SVG elements are not supported, so temporary
  image files are created with associated image links.

### Temporary files

Since the `svgdx-pandoc` filter will exit prior to the final output being rendered,
it cannot clean up after itself, and any temporary files created in order to create
e.g. a PDF file will remain, typically in the directory given by the `TMPDIR` environment
variable or other system-specific location.

If necessary, the location of temporary rendered SVG images can be controlled by defining
the `SVGDX_PANDOC_TMPDIR` environment variable, which should point to a directory which
already exists.

## Developing

To test changes to `svgdx-pandoc`, run as follows:

```
cargo build && pandoc --filter target/debug/svgdx-pandoc [input] -o [output]
```

In order to test changes to the `svgdx` library itself, update the appropriate `dependencies`
entry of [Cargo.toml](Cargo.toml) of this (svgdx-pandoc) repo to point to a local clone of `svgdx`,
rather than providing a version specifier:

```toml
svgdx = { path = "/path/to/svgdx", default-features = false }
```

## License

This repository is released under the MIT license; for more information see the [LICENSE](LICENSE) file.
