pub enum TransformResult {
    Svg(String),
    Error(String),
}

fn error_format(s: &str) -> String {
    format!(
        r#"<div style="color: red; border: 5px double red; padding: 1em;">{}</div>"#,
        s.replace('\n', "<br/>")
    )
}

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

pub fn svgdx_handler(s: &str) -> TransformResult {
    match svgdx::transform_str_default(s.to_string()) {
        Ok(s) => TransformResult::Svg(blank_line_remover(&s)),
        Err(e) => TransformResult::Error(error_format(&e.to_string())),
    }
}
