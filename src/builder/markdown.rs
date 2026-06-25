//! Markdown → HTML rendering via comrak, plus `.md` → `.html` link rewriting.
//!
//! Replaces Goldmark + goldmark-mathjax. Link rewriting is done on the parsed
//! AST (the Go version used a regex in `preprocessMarkdown`), so it only touches
//! real link destinations and correctly handles `.md#anchor`.

use comrak::nodes::{AstNode, NodeValue};
use comrak::{Arena, Options, format_html, parse_document};

/// Comrak options shared by parse + render: GFM-ish extensions and raw-HTML
/// passthrough (Goldmark's GFM allowed inline HTML and embedded images).
fn options() -> Options<'static> {
    let mut o = Options::default();
    o.extension.strikethrough = true;
    o.extension.table = true;
    o.extension.autolink = true;
    o.extension.tasklist = true;
    o.extension.footnotes = true;
    o.extension.math_dollars = true;
    o.parse.smart = false;
    o.render.r#unsafe = true; // allow raw HTML / image tags through
    o
}

/// Render markdown to an HTML fragment with links rewritten and math converted
/// to MathJax-standard delimiters.
pub fn render(markdown: &str) -> String {
    let opts = options();
    let arena = Arena::new();
    let root = parse_document(&arena, markdown, &opts);
    transform(root);
    let mut html = String::new();
    format_html(root, &opts, &mut html)
        .expect("comrak html formatting into a String is infallible");
    html
}

fn transform<'a>(node: &'a AstNode<'a>) {
    for child in node.children() {
        transform(child);
    }
    let mut data = node.data.borrow_mut();
    match &mut data.value {
        NodeValue::Link(link) => {
            link.url = rewrite_md_url(&link.url);
        }
        // comrak emits math as `<span data-math-style=...>` which MathJax does
        // not process. Re-emit raw `\(...\)` / `\[...\]` so MathJax handles it;
        // the literal is not markdown-interpreted, so it is reproduced verbatim.
        NodeValue::Math(math) => {
            let wrapped = if math.display_math {
                format!("\\[{}\\]", math.literal)
            } else {
                format!("\\({}\\)", math.literal)
            };
            data.value = NodeValue::Raw(wrapped);
        }
        _ => {}
    }
}

/// `foo.md` → `foo.html`, `foo.md#bar` → `foo.html#bar`. External/non-.md URLs
/// are untouched.
fn rewrite_md_url(url: &str) -> String {
    if let Some(stem) = url.strip_suffix(".md") {
        format!("{stem}.html")
    } else if let Some(pos) = url.find(".md#") {
        let mut s = url.to_string();
        s.replace_range(pos..pos + 3, ".html");
        s
    } else {
        url.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rewrites_md_links() {
        assert_eq!(rewrite_md_url("notes.md"), "notes.html");
        assert_eq!(rewrite_md_url("dir/notes.md#h"), "dir/notes.html#h");
        assert_eq!(rewrite_md_url("https://x.com/a"), "https://x.com/a");
        assert_eq!(rewrite_md_url("image.png"), "image.png");
    }

    #[test]
    fn renders_link_rewrite_in_html() {
        let html = render("[see](other.md)");
        assert!(html.contains("href=\"other.html\""), "got: {html}");
    }

    #[test]
    fn renders_gfm_table_and_strikethrough() {
        let html = render("~~x~~\n\n| a | b |\n|---|---|\n| 1 | 2 |\n");
        assert!(html.contains("<del>"), "got: {html}");
        assert!(html.contains("<table>"), "got: {html}");
    }

    #[test]
    fn math_uses_mathjax_delimiters() {
        let inline = render("mass $E = mc^2$ done");
        assert!(inline.contains("\\(E = mc^2\\)"), "got: {inline}");
        let display = render("$$\\int_0^1 x\\,dx$$");
        assert!(display.contains("\\[\\int_0^1 x\\,dx\\]"), "got: {display}");
        // Underscores inside math must survive (not become markdown emphasis).
        let sub = render("$x_1 + x_2$");
        assert!(sub.contains("\\(x_1 + x_2\\)"), "got: {sub}");
    }
}
