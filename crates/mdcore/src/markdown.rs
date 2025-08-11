use comrak::{markdown_to_html, ComrakOptions};
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref MATH_INLINE: Regex = Regex::new(r"\$([^\$
]+)\$")
        .expect("Invalid MATH_INLINE regex pattern");
    static ref MATH_BLOCK: Regex = Regex::new(r"\$\$([^\$]+)\$\$")
        .expect("Invalid MATH_BLOCK regex pattern");
    static ref MERMAID_BLOCK: Regex =
        Regex::new(r#"<pre><code class="language-mermaid">([^<]*)</code></pre>"#)
            .expect("Invalid MERMAID_BLOCK regex pattern");
}

pub fn to_html(src: &str) -> String {
    let opt = create_comrak_options();
    let mut html = markdown_to_html(src, &opt);
    html = patch_math_blocks(&html);
    html = patch_mermaid_blocks(&html);
    html
}

fn create_comrak_options() -> ComrakOptions<'static> {
    let mut opt = ComrakOptions::default();

    // Extension options
    opt.extension.strikethrough = true;
    opt.extension.table = true;
    opt.extension.autolink = true;
    opt.extension.tasklist = true;
    opt.extension.superscript = true;
    opt.extension.footnotes = true;
    opt.extension.description_lists = true;

    // Parse options
    opt.parse.smart = true;

    // Render options - SECURITY: Enable safe HTML rendering
    opt.render.unsafe_ = false;  // Disable unsafe HTML execution
    opt.render.escape = true;    // Enable HTML escaping to prevent XSS

    opt
}

pub fn patch_math_blocks(html: &str) -> String {
    // Process block math first ($$...$$)
    let result = MATH_BLOCK
        .replace_all(
            html,
            r#"<div class="math-block" data-math="$1">$$1$</div>"#,
        );

    // Process inline math ($...$)
    MATH_INLINE
        .replace_all(
            &result,
            r#"<span class="math-inline" data-math="$1">$$$1$$</span>"#,
        )
        .into_owned()
}

pub fn patch_mermaid_blocks(html: &str) -> String {
    MERMAID_BLOCK
        .replace_all(html, r#"<div class="mermaid">$1</div>"#)
        .into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_markdown() {
        let md = "# Hello\n\nThis is **bold** and *italic*.";
        let html = to_html(md);
        assert!(html.contains("<h1>"));
        assert!(html.contains("<strong>"));
        assert!(html.contains("<em>"));
    }

    #[test]
    fn test_gfm_table() {
        let md = "| Header 1 | Header 2 |\n|----------|----------|\n| Cell 1   | Cell 2   |";
        let html = to_html(md);
        assert!(html.contains("<table>"));
        assert!(html.contains("<thead>"));
        assert!(html.contains("<tbody>"));
    }

    #[test]
    fn test_math_inline() {
        let md = "This is inline math: $x^2 + y^2 = z^2$.";
        let html = to_html(md);
        assert!(html.contains(r#"class="math-inline""#));
    }

    #[test]
    fn test_math_block() {
        let md = "$$\n\\int_0^1 x^2 dx = \\frac{1}{3}\n$$";
        let html = to_html(md);
        assert!(html.contains(r#"class="math-block""#));
    }

    #[test]
    fn test_mermaid() {
        let md = "```mermaid\ngraph LR\n  A --> B\n```";
        let html = to_html(md);
        assert!(html.contains(r#"class="mermaid""#));
    }
}
