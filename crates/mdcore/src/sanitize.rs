use ammonia::Builder;

pub fn sanitize_html(html: &str) -> String {
    // Use safe defaults - no script tags allowed
    create_secure_sanitizer().clean(html).to_string()
}

pub fn sanitize_with_options(html: &str, allow_math_classes: bool) -> String {
    // SECURITY: Remove dangerous allow_scripts parameter
    // Always use secure sanitization with optional math support
    if allow_math_classes {
        // Allow math-specific classes for KaTeX rendering
        Builder::new()
            .add_allowed_classes("span", &["math-inline", "math-block"])
            .add_allowed_classes("div", &["math-block", "mermaid"])
            .clean(html)
            .to_string()
    } else {
        create_secure_sanitizer().clean(html).to_string()
    }
}

fn create_secure_sanitizer() -> Builder<'static> {
    // Create a default sanitizer with safe settings
    // ammonia's default settings are already quite secure
    Builder::new()
}
