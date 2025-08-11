pub mod markdown;
pub mod sanitize;

pub use markdown::to_html;

#[cfg(test)]
mod tests;
