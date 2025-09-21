#[cfg(test)]
mod unit_tests {
    use super::super::*;

    #[test]
    fn test_sanitize_html_removes_scripts() {
        let html = r#"<p>Hello</p><script>alert('XSS')</script><p>World</p>"#;
        let sanitized = sanitize::sanitize_html(html);
        assert!(!sanitized.contains("<script"));
        assert!(!sanitized.contains("alert"));
        assert!(sanitized.contains("Hello"));
        assert!(sanitized.contains("World"));
    }

    #[test]
    fn test_sanitize_with_math_classes() {
        let html = r#"<span class="math-inline">x^2</span><div class="mermaid">graph TD</div>"#;
        let sanitized = sanitize::sanitize_with_options(html, true);
        assert!(sanitized.contains("math-inline"));
        assert!(sanitized.contains("mermaid"));
    }

    #[test]
    fn test_sanitize_removes_dangerous_attributes() {
        let html = r#"<a href="javascript:alert('XSS')">Click me</a>"#;
        let sanitized = sanitize::sanitize_html(html);
        assert!(!sanitized.contains("javascript:"));
    }

    #[test]
    fn test_markdown_to_html_basic() {
        let markdown = "# Hello\n\nThis is **bold** text.";
        let html = markdown::to_html(markdown);
        assert!(html.contains("<h1>"));
        assert!(html.contains("<strong>"));
    }

    #[test]
    fn test_markdown_table_rendering() {
        let markdown = r#"
| Header 1 | Header 2 |
|----------|----------|
| Cell 1   | Cell 2   |
"#;
        let html = markdown::to_html(markdown);
        assert!(html.contains("<table>"));
        assert!(html.contains("<th>"));
        assert!(html.contains("<td>"));
    }

    #[test]
    fn test_patch_math_blocks() {
        let html = r#"<p>Inline $x^2$ and display $$y = mx + b$$</p>"#;
        let patched = markdown::patch_math_blocks(html);
        assert!(patched.contains(r#"class="math-inline""#));
        assert!(patched.contains(r#"class="math-block""#));
    }

    #[test]
    fn test_patch_mermaid_blocks() {
        let html = r#"<pre><code class="language-mermaid">graph TD</code></pre>"#;
        let patched = markdown::patch_mermaid_blocks(html);
        assert!(patched.contains(r#"class="mermaid""#));
        assert!(!patched.contains("<pre>"));
        assert!(!patched.contains("<code"));
    }

    #[test]
    fn test_markdown_security_escape() {
        let markdown = "<script>alert('XSS')</script>";
        let html = markdown::to_html(markdown);
        let sanitized = sanitize::sanitize_html(&html);
        assert!(!sanitized.contains("<script"));
        // The text "alert" might still appear as escaped text content
        // We mainly care that the script tag is removed
        assert!(!sanitized.contains("<script>"));
    }
}
