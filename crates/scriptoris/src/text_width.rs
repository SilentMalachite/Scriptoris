//! クロスプラットフォーム対応の文字幅計算モジュール
//!
//! 日本語・東アジア文字の正確な表示幅計算と、
//! 各プラットフォームでの互換性を提供します。

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthChar;

/// 文字幅計算用の構造体
#[derive(Debug, Clone)]
pub struct TextWidthCalculator {
    /// East Asian Widthプロパティを考慮するかどうか
    east_asian_aware: bool,
    /// 絵文字の幅をどう扱うか
    emoji_width: EmojiWidth,
}

/// 絵文字の幅扱い
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmojiWidth {
    /// 幅1として扱う
    #[allow(dead_code)]
    One,
    /// 幅2として扱う
    #[allow(dead_code)]
    Two,
    /// Unicode標準の幅を使用
    Standard,
}

impl Default for TextWidthCalculator {
    fn default() -> Self {
        Self {
            east_asian_aware: true,
            emoji_width: EmojiWidth::Standard,
        }
    }
}

impl TextWidthCalculator {
    /// 新しい文字幅計算器を作成
    pub fn new() -> Self {
        Self::default()
    }

    /// East Asian Widthの設定
    pub fn east_asian_aware(mut self, aware: bool) -> Self {
        self.east_asian_aware = aware;
        self
    }

    /// 絵文字幅の設定
    pub fn emoji_width(mut self, width: EmojiWidth) -> Self {
        self.emoji_width = width;
        self
    }

    /// 文字列の表示幅を計算
    pub fn str_width(&self, text: &str) -> usize {
        if text.is_empty() {
            return 0;
        }

        // Unicodeセグメントに分割して処理
        text.graphemes(true).map(|g| self.grapheme_width(g)).sum()
    }

    /// グラフェムクラスタの表示幅を計算
    pub fn grapheme_width(&self, grapheme: &str) -> usize {
        if grapheme.is_empty() {
            return 0;
        }

        // 制御文字は幅0
        if grapheme.chars().all(|c| c.is_control()) {
            return 0;
        }

        // 絵文字の処理
        if self.is_emoji(grapheme) {
            return match self.emoji_width {
                EmojiWidth::One => 1,
                EmojiWidth::Two => 2,
                EmojiWidth::Standard => {
                    // 絵文字の標準的な幅判定
                    if grapheme.chars().count() > 1 {
                        // 結合絵文字の場合
                        2
                    } else {
                        // シングル絵文字の場合、unicode_widthに従う
                        grapheme.chars().next().and_then(|c| c.width()).unwrap_or(1)
                    }
                }
            };
        }

        // 通常の文字はunicode_width crateを使用
        // East Asianが有効な場合、全角・半角を正確に判定
        if self.east_asian_aware {
            self.calculate_east_asian_width(grapheme)
        } else {
            grapheme.chars().map(|c| c.width().unwrap_or(1)).sum()
        }
    }

    /// 東アジア文字幅の計算
    fn calculate_east_asian_width(&self, grapheme: &str) -> usize {
        let mut total_width = 0;
        let mut chars = grapheme.chars().peekable();

        while let Some(c) = chars.next() {
            let width = if self.is_fullwidth_char(c) {
                2
            } else if self.is_wide_emoji_sequence(c, &mut chars) {
                // 絵文字シーケンスの特別処理
                match self.emoji_width {
                    EmojiWidth::One => 1,
                    EmojiWidth::Two => 2,
                    EmojiWidth::Standard => 2,
                }
            } else {
                c.width().unwrap_or(1)
            };
            total_width += width;
        }

        total_width
    }

    /// 全角文字かどうかを判定
    fn is_fullwidth_char(&self, c: char) -> bool {
        // Unicode East Asian WidthがW（全角）またはF（全幅）の文字
        match c.width().unwrap_or(1) {
            2 => true,
            1 => false,
            _ => {
                // 不明な場合は文字コードで判定
                self.is_fullwidth_by_codepoint(c)
            }
        }
    }

    /// 文字コードベースでの全角判定（フォールバック）
    fn is_fullwidth_by_codepoint(&self, c: char) -> bool {
        let code = c as u32;

        // CJK統合漢字
        (0x4E00..=0x9FFF).contains(&code) ||
        // CJK統合漢字拡張A
        (0x3400..=0x4DBF).contains(&code) ||
        // CJK統合漢字拡張B
        (0x20000..=0x2A6DF).contains(&code) ||
        // CJK互換漢字
        (0xF900..=0xFAFF).contains(&code) ||
        // ひらがな
        (0x3040..=0x309F).contains(&code) ||
        // カタカナ
        (0x30A0..=0x30FF).contains(&code) ||
        // ハングル音節
        (0xAC00..=0xD7AF).contains(&code) ||
        // 全角記号・全角英数 (半角カタカナを除く)
        ((0xFF00..=0xFFEF).contains(&code) && !(0xFF61..=0xFF9F).contains(&code))
    }

    /// 絵文字かどうかを判定
    fn is_emoji(&self, text: &str) -> bool {
        // 絵文字のUnicode範囲とシーケンスを判定
        text.chars().any(|c| {
            let code = c as u32;
            // 絵文字の基本範囲
            (0x1F600..=0x1F64F).contains(&code) || // Emoticons
            (0x1F300..=0x1F5FF).contains(&code) || // Misc Symbols and Pictographs
            (0x1F680..=0x1F6FF).contains(&code) || // Transport and Map
            (0x1F1E0..=0x1F1FF).contains(&code) || // Regional Indicator Symbols
            (0x2600..=0x26FF).contains(&code) ||   // Misc Symbols
            (0x2700..=0x27BF).contains(&code) ||   // Dingbats
            // その他の絵文字
            self.is_variation_selector(c) ||
            self.is_zero_width_joiner(c)
        }) || text.contains("🏻")
            || text.contains("🏼")
            || text.contains("🏽")
            || text.contains("🏾")
            || text.contains("🏿")
    }

    /// バリエーションセレクタかどうか
    fn is_variation_selector(&self, c: char) -> bool {
        matches!(c, '\u{FE00}'..='\u{FE0F}')
    }

    /// ゼロ幅結合子かどうか
    fn is_zero_width_joiner(&self, c: char) -> bool {
        c == '\u{200D}'
    }

    /// 幅の広い絵文字シーケンスかどうか
    fn is_wide_emoji_sequence(
        &self,
        c: char,
        chars: &mut std::iter::Peekable<std::str::Chars>,
    ) -> bool {
        if c == '\u{200D}' {
            // ZWJの後に続く文字を確認
            if let Some(&next_c) = chars.peek() {
                let code = next_c as u32;
                (0x1F600..=0x1F64F).contains(&code) || // Emoticons
                (0x1F300..=0x1F5FF).contains(&code) || // Misc Symbols
                (0x1F680..=0x1F6FF).contains(&code) // Transport
            } else {
                false
            }
        } else {
            false
        }
    }

    /// 指定されたカラム位置に対応する文字インデックスを取得
    pub fn col_to_char_index(&self, text: &str, display_col: usize) -> usize {
        let mut current_width = 0;
        let mut char_index = 0;

        for grapheme in text.graphemes(true) {
            let grapheme_width = self.grapheme_width(grapheme);

            if current_width + grapheme_width > display_col {
                break;
            }

            current_width += grapheme_width;
            char_index += grapheme.len();
        }

        char_index.min(text.len())
    }

    /// 指定された文字インデックスに対応する表示カラム位置を取得
    pub fn char_index_to_col(&self, text: &str, char_index: usize) -> usize {
        let mut current_width = 0;
        let mut processed_chars = 0;

        for grapheme in text.graphemes(true) {
            if processed_chars >= char_index {
                break;
            }

            current_width += self.grapheme_width(grapheme);
            processed_chars += grapheme.len();
        }

        current_width
    }
}

/// 文字列を指定された幅で折り返す
#[allow(dead_code)]
pub fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    let calculator = TextWidthCalculator::new();
    let mut lines = Vec::new();
    let mut current_line = String::new();
    let mut current_width = 0;

    for grapheme in text.graphemes(true) {
        let grapheme_width = calculator.grapheme_width(grapheme);

        if grapheme == "\n" {
            lines.push(current_line.clone());
            current_line.clear();
            current_width = 0;
        } else if current_width + grapheme_width > max_width {
            // 折り返し位置を検索
            if let Some(wrap_pos) = find_wrap_position(&current_line, max_width) {
                lines.push(current_line[..wrap_pos].to_string());
                current_line = current_line[wrap_pos..].to_string();
                current_line.push_str(grapheme);
                current_width = calculator.str_width(&current_line);
            } else {
                lines.push(current_line.clone());
                current_line = grapheme.to_string();
                current_width = grapheme_width;
            }
        } else {
            current_line.push_str(grapheme);
            current_width += grapheme_width;
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    lines
}

/// 適切な折り返し位置を見つける
#[allow(dead_code)]
fn find_wrap_position(text: &str, max_width: usize) -> Option<usize> {
    let calculator = TextWidthCalculator::new();
    let mut best_pos = None;
    let mut current_width = 0;

    for (i, grapheme) in text.graphemes(true).enumerate() {
        let grapheme_width = calculator.grapheme_width(grapheme);
        current_width += grapheme_width;

        if current_width > max_width {
            break;
        }

        // 空白や区切り文字を折り返し位置として好む
        if grapheme.trim().is_empty() {
            best_pos = Some(i + grapheme.len());
        }
    }

    best_pos.or_else(|| {
        // 適切な位置が見つからない場合は文字単位で分割
        let mut width = 0;
        for (i, c) in text.char_indices() {
            width += c.width().unwrap_or(1);
            if width > max_width {
                return Some(i);
            }
        }
        None
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_width_calculator() {
        let calc = TextWidthCalculator::new();

        // ASCII文字
        assert_eq!(calc.str_width("Hello"), 5);

        // 日本語文字（全角）
        assert_eq!(calc.str_width("こんにちは"), 10);

        // 混在テキスト
        assert_eq!(calc.str_width("Hello世界"), 9);
    }

    #[test]
    fn test_grapheme_width() {
        let calc = TextWidthCalculator::new();

        // 基本文字
        assert_eq!(calc.grapheme_width("a"), 1);
        assert_eq!(calc.grapheme_width("あ"), 2);

        // 絵文字
        assert_eq!(calc.grapheme_width("😀"), 2);
        assert_eq!(calc.grapheme_width("👍"), 2);
    }

    #[test]
    fn test_col_to_char_index() {
        let calc = TextWidthCalculator::new();
        let text = "Hello世界";

        // Test the actual widths first
        // "Hello" = 5 ASCII chars, width 5
        // "世" = 1 char, width 2 (fullwidth)
        // "界" = 1 char, width 2 (fullwidth)
        // Total: 7 chars, width 9

        assert_eq!(calc.str_width("Hello"), 5);
        assert_eq!(calc.str_width("世"), 2);
        assert_eq!(calc.str_width("界"), 2);
        assert_eq!(calc.str_width(text), 9);

        // ASCII部分 (width 1 each)
        assert_eq!(calc.col_to_char_index(text, 3), 3);

        // 日本語部分
        // col 5 = after "Hello", at start of "世" (byte index 5)
        assert_eq!(calc.col_to_char_index(text, 5), 5);
        // col 7 = after "Hello世" (5+2), at start of "界" (byte index 8)
        assert_eq!(calc.col_to_char_index(text, 7), 8);
    }

    #[test]
    fn test_wrap_text() {
        let text = "Hello世界、これはテストです。";
        let lines = wrap_text(text, 10);
        let calc = TextWidthCalculator::new();

        assert!(lines.len() > 1);
        // Check display width, not byte length
        for line in &lines {
            let width = calc.str_width(line);
            assert!(width <= 10, "Line '{}' has width {} > 10", line, width);
        }
    }
}
