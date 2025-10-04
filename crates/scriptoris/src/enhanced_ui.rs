use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, Mode};

pub struct EnhancedUI;

impl EnhancedUI {
    pub fn draw(f: &mut Frame, app: &mut App) {
        // Enhanced UI draw with comprehensive error handling
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            Self::draw_internal(f, app)
        }));

        if let Err(panic_info) = result {
            log::error!("Enhanced UI panic: {:?}", panic_info);
            Self::draw_fallback_error(f, f.size(), "拡張UIでパニックが発生しました");
            return;
        }

        // Validate frame size first
        if f.size().width == 0 || f.size().height == 0 {
            log::error!("Invalid frame size in enhanced UI: {:?}", f.size());
            Self::draw_fallback_error(f, f.size(), "フレームサイズが無効です");
            return;
        }

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1), // Title bar
                    Constraint::Min(0),    // Editor area
                    Constraint::Length(3), // Enhanced status bar
                ])
                .split(f.size());

            // Validate chunk creation
            if chunks.len() != 3 {
                log::error!("Expected 3 chunks in enhanced UI, got {}", chunks.len());
                return Err("チャンク作成に失敗しました");
            }

            // Draw title bar
            Self::draw_enhanced_title_bar(f, app, chunks[0]);

            // Draw main content area
            if app.show_help() {
                Self::draw_enhanced_help(f, chunks[1]);
            } else {
                Self::draw_enhanced_editor(f, app, chunks[1]);
            }

            // Draw enhanced status bar
            Self::draw_enhanced_status_bar(f, app, chunks[2]);

            Ok::<(), &'static str>(())
        }));

        if let Err(error_info) = result {
            let error_msg = match error_info.downcast::<&'static str>() {
                Ok(msg) => *msg,
                Err(_) => "拡張UIで不明なエラーが発生しました",
            };
            log::error!("Enhanced UI error: {}", error_msg);
            Self::draw_fallback_error(f, f.size(), error_msg);
        }
    }

    fn draw_internal(f: &mut Frame, _app: &mut App) -> Result<(), &'static str> {
        // Additional validation
        if f.size().width < 10 || f.size().height < 3 {
            return Err("ターミナルサイズが小さすぎます");
        }

        Ok(())
    }

    fn draw_enhanced_title_bar(f: &mut Frame, app: &App, area: Rect) {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let file_info = match app.file_path() {
                Some(path) => {
                    let filename = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("Unknown");
                    let dir = path.parent().and_then(|p| p.to_str()).unwrap_or("");
                    format!("  {} • {}", filename, dir)
                }
                None => String::from("  [新規ファイル]"),
            };

            let modified_indicator = if app.is_modified() {
                " ●(変更あり)"
            } else {
                ""
            };
            let title = format!("Scriptoris{}{}", modified_indicator, file_info);

            // Color scheme based on modification state
            let style = if app.is_modified() {
                Style::default().bg(Color::Red).fg(Color::White)
            } else {
                Style::default().bg(Color::Blue).fg(Color::White)
            };

            let title_bar = Paragraph::new(title)
                .style(style)
                .alignment(Alignment::Left);

            f.render_widget(title_bar, area);
        }));

        if result.is_err() {
            // Fallback to simple title bar
            Self::draw_fallback_title_bar(f, app, area);
        }
    }

    fn draw_enhanced_editor(f: &mut Frame, app: &mut App, area: Rect) {
        let editor_area = if app.config.editor.line_numbers {
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Length(5), // Line numbers
                    Constraint::Min(0),    // Editor content
                ])
                .split(area)
        } else {
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Min(0)])
                .split(area)
        };

        app.get_current_editor_mut()
            .set_viewport_height(area.height as usize);

        // Draw line numbers if enabled
        if app.config.editor.line_numbers {
            let start_line = app
                .get_current_editor()
                .cursor_position()
                .0
                .saturating_sub(area.height as usize / 2);
            let line_numbers: Vec<Line> = (0..area.height as usize)
                .map(|i| {
                    let line_num = start_line + i + 1;
                    let style = if line_num == app.get_current_editor().cursor_position().0 + 1 {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::DarkGray)
                    };
                    Line::from(Span::styled(format!("{:4} ", line_num), style))
                })
                .collect();

            let line_number_widget = Paragraph::new(line_numbers)
                .style(Style::default().bg(Color::Black))
                .alignment(Alignment::Right);

            f.render_widget(line_number_widget, editor_area[0]);
        }

        // Draw editor content with syntax highlighting
        let lines = app.get_current_editor().get_viewport_lines();
        let (cursor_line, _cursor_col) = app.get_current_editor().cursor_position();

        // Get file path before borrowing highlighter
        let file_path = app.file_path().map(|p| p.to_string_lossy().to_string());
        let highlighter = app.get_highlighter();
        let syntax = match file_path.as_ref() {
            Some(p) => highlighter.find_syntax_for_filename(p),
            None => highlighter.find_syntax_for_filename("text.md"),
        };

        let mut content_lines = highlighter.highlight_lines_to_ratatui(&lines, syntax);

        // Current line background highlight overlay
        if app.config.editor.highlight_current_line && cursor_line < content_lines.len() {
            let bg = Style::default().bg(Color::Rgb(40, 40, 40));
            let spans = content_lines[cursor_line]
                .spans
                .iter()
                .map(|s| Span::styled(s.content.clone().into_owned(), s.style.patch(bg)))
                .collect::<Vec<_>>();
            content_lines[cursor_line] = Line::from(spans);
        }

        let editor_widget = Paragraph::new(content_lines)
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: false });

        let editor_area_index = if app.config.editor.line_numbers { 1 } else { 0 };
        f.render_widget(editor_widget, editor_area[editor_area_index]);

        // Draw cursor
        if let Some(editor_rect) = editor_area.get(editor_area_index) {
            Self::draw_cursor(f, app, *editor_rect);
        }
    }

    fn draw_cursor(f: &mut Frame, app: &App, area: Rect) {
        let (cursor_line, cursor_col) = app.get_current_editor().cursor_position();

        // Compute display column considering fullwidth characters on the line
        // We recompute width from start to cursor_col for correctness
        let line_text = {
            let lines = app.get_current_editor().get_viewport_lines();
            lines.get(cursor_line).cloned().unwrap_or_default()
        };
        let logical_prefix: String = line_text.chars().take(cursor_col).collect();

        // Use accurate text width calculation for cross-platform compatibility
        let display_col: usize = app.text_calculator.str_width(&logical_prefix);

        // Calculate cursor position on screen
        if cursor_line < area.height as usize && display_col < area.width as usize {
            let cursor_x = area.x + display_col as u16;
            let cursor_y = area.y + cursor_line as u16;

            if cursor_x < area.x + area.width && cursor_y < area.y + area.height {
                // Draw cursor based on mode
                let cursor_style = match app.mode() {
                    Mode::Insert => Style::default().bg(Color::Green),
                    Mode::Normal => Style::default().bg(Color::Gray),
                    Mode::Command => Style::default().bg(Color::Yellow),
                    _ => Style::default().bg(Color::White),
                };

                let cursor_area = Rect {
                    x: cursor_x,
                    y: cursor_y,
                    width: 1,
                    height: 1,
                };
                let cursor = Paragraph::new(" ").style(cursor_style);
                f.render_widget(cursor, cursor_area);
            }
        }
    }

    fn draw_enhanced_status_bar(f: &mut Frame, app: &App, area: Rect) {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1), // File info and cursor position
                    Constraint::Length(1), // Command input or shortcuts
                    Constraint::Length(1), // Status messages
                ])
                .split(area);

            // Draw file info and cursor position
            Self::draw_file_info(f, app, chunks[0]);

            // Draw command input or shortcuts
            Self::draw_command_area(f, app, chunks[1]);

            // Draw status messages
            Self::draw_status_messages(f, app, chunks[2]);
        }));

        if result.is_err() {
            // Fallback to simple status bar
            Self::draw_fallback_status_bar(f, app, area);
        }
    }

    fn draw_file_info(f: &mut Frame, app: &App, area: Rect) {
        let (line, col) = app.get_current_editor().cursor_position();
        let line_count = app.get_current_editor().line_count();
        let progress = if line_count > 0 {
            line * 100 / line_count.max(1)
        } else {
            0
        };

        let file_info = match app.file_path() {
            Some(path) => format!("{} ", path.display()),
            None => "[新規ファイル] ".to_string(),
        };

        let position_info = format!(
            "行 {}, 桁 {} ({}/{})",
            line + 1,
            col + 1,
            line + 1,
            line_count
        );
        let progress_info = format!(" {}%", progress);

        let info_spans = vec![
            Span::styled(file_info, Style::default().fg(Color::Cyan)),
            Span::styled(position_info, Style::default().fg(Color::White)),
            Span::styled(progress_info, Style::default().fg(Color::Green)),
        ];

        let info_line = Paragraph::new(Line::from(info_spans))
            .style(Style::default().bg(Color::DarkGray))
            .alignment(Alignment::Left);

        f.render_widget(info_line, area);
    }

    fn draw_command_area(f: &mut Frame, app: &App, area: Rect) {
        match app.mode() {
            Mode::Command => {
                let input =
                    Paragraph::new(format!("{}{}", app.status_message(), app.command_buffer()))
                        .style(Style::default().fg(Color::Yellow).bg(Color::Black));
                f.render_widget(input, area);
            }
            _ => {
                let shortcuts = vec![
                    Span::styled(
                        ":",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" コマンド  "),
                    Span::styled(
                        "i",
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" 挿入  "),
                    Span::styled(
                        "/",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" 検索  "),
                    Span::styled(
                        "u",
                        Style::default()
                            .fg(Color::Magenta)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" 元に戻す  "),
                    Span::styled(
                        "^R",
                        Style::default()
                            .fg(Color::Magenta)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" やり直し  "),
                    Span::styled(
                        "?",
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" ヘルプ"),
                ];

                let shortcut_bar = Paragraph::new(Line::from(shortcuts))
                    .style(Style::default().bg(Color::DarkGray));
                f.render_widget(shortcut_bar, area);
            }
        }
    }

    fn draw_status_messages(f: &mut Frame, app: &App, area: Rect) {
        // Get the current status message
        let status_text = app.status_message().to_string();

        if !status_text.is_empty() {
            // Determine message type from content (simple heuristic)
            let (style, prefix) = if status_text.contains("Error") || status_text.contains("error")
            {
                (
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                    "✖ ",
                )
            } else if status_text.contains("saved")
                || status_text.contains("Wrote")
                || status_text.contains("pasted")
            {
                (Style::default().fg(Color::Green), "✓ ")
            } else if status_text.contains("Warning") || status_text.contains("No write since") {
                (Style::default().fg(Color::Yellow), "⚠ ")
            } else if status_text.contains("Searching") || status_text.contains("Found") {
                (Style::default().fg(Color::Cyan), "🔍 ")
            } else if status_text.contains("Undone") || status_text.contains("Redone") {
                (Style::default().fg(Color::Magenta), "⟲ ")
            } else {
                (Style::default().fg(Color::White), "ℹ ")
            };

            let message = format!("{}{}", prefix, status_text);
            let status_widget = Paragraph::new(message)
                .style(style)
                .alignment(Alignment::Left);

            f.render_widget(status_widget, area);
        } else {
            // Show mode indicator when no status message
            let mode_text = match app.mode() {
                Mode::Normal => "-- NORMAL --",
                Mode::Insert => "-- INSERT --",
                Mode::Command => "-- COMMAND --",
                Mode::Visual => "-- VISUAL --",
                Mode::VisualBlock => "-- VISUAL BLOCK --",
                Mode::Replace => "-- REPLACE --",
                Mode::Help => "-- HELP --",
                Mode::SavePrompt => "-- SAVE PROMPT --",
            };

            let mode_style = match app.mode() {
                Mode::Normal => Style::default().fg(Color::Blue),
                Mode::Insert => Style::default().fg(Color::Green),
                Mode::Command => Style::default().fg(Color::Yellow),
                Mode::Visual => Style::default().fg(Color::Magenta),
                Mode::VisualBlock => Style::default().fg(Color::Cyan),
                Mode::Replace => Style::default().fg(Color::Red),
                Mode::Help => Style::default().fg(Color::Cyan),
                Mode::SavePrompt => Style::default().fg(Color::Red),
            };

            let mode_widget = Paragraph::new(mode_text)
                .style(mode_style.add_modifier(Modifier::BOLD))
                .alignment(Alignment::Center);

            f.render_widget(mode_widget, area);
        }
    }

    fn draw_enhanced_help(f: &mut Frame, area: Rect) {
        let help_text = vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                " SCRIPTORIS - Enhanced Vim-style Markdown Editor",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                " Mode Commands:",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::UNDERLINED),
            )]),
            Line::from(vec![
                Span::styled(
                    "   i",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("   Enter Insert Mode        "),
                Span::styled(
                    "Esc",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::raw("   Return to Normal Mode"),
            ]),
            Line::from(vec![
                Span::styled(
                    "   :",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("   Enter Command Mode       "),
                Span::styled(
                    "/",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("     Search Text"),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                " Movement:",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::UNDERLINED),
            )]),
            Line::from(vec![
                Span::styled(
                    "   h j k l",
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("   Move cursor (Vim-style)"),
            ]),
            Line::from(vec![
                Span::styled(
                    "   Arrow Keys",
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("   Move cursor (Traditional)"),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                " Editing:",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::UNDERLINED),
            )]),
            Line::from(vec![
                Span::styled(
                    "   u",
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("   Undo                     "),
                Span::styled(
                    "Ctrl+R",
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("   Redo"),
            ]),
            Line::from(vec![
                Span::styled(
                    "   x",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::raw("   Delete character         "),
                Span::styled(
                    "d",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::raw("      Delete line"),
            ]),
            Line::from(vec![
                Span::styled(
                    "   p",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("   Paste                    "),
                Span::styled(
                    "o",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("      New line below"),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                " Commands:",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::UNDERLINED),
            )]),
            Line::from(vec![
                Span::styled(
                    "   :w",
                    Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("   Save file                "),
                Span::styled(
                    ":q",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::raw("     Quit"),
            ]),
            Line::from(vec![
                Span::styled(
                    "   :wq",
                    Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("  Save and quit            "),
                Span::styled(
                    ":q!",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::raw("    Force quit"),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(" Press ", Style::default().fg(Color::Gray)),
                Span::styled(
                    "?",
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" or ", Style::default().fg(Color::Gray)),
                Span::styled(
                    "Esc",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::styled(" to close this help", Style::default().fg(Color::Gray)),
            ]),
        ];

        let help_widget = Paragraph::new(help_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(Span::styled(
                        "Help",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ))
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .wrap(Wrap { trim: false })
            .alignment(Alignment::Left);

        f.render_widget(help_widget, area);
    }

    // Fallback UI methods for enhanced UI error recovery - public for external use
    pub fn draw_fallback_title_bar(f: &mut Frame, _app: &App, area: Rect) {
        let title = "Scriptoris Enhanced - エラー回復";
        let title_bar = Paragraph::new(title)
            .style(Style::default().bg(Color::Red).fg(Color::White))
            .alignment(Alignment::Left);
        f.render_widget(title_bar, area);
    }

    pub fn draw_fallback_status_bar(f: &mut Frame, _app: &App, area: Rect) {
        let status = "拡張UIエラー: 基本表示モードに切り替えてください";
        let status_bar = Paragraph::new(status)
            .style(Style::default().bg(Color::DarkGray).fg(Color::Yellow))
            .alignment(Alignment::Center);
        f.render_widget(status_bar, area);
    }

    pub fn draw_fallback_error(f: &mut Frame, area: Rect, message: &str) {
        let error_text = vec![
            Line::from(""),
            Line::from("拡張UIエラーが発生しました"),
            Line::from(""),
            Line::from(format!("詳細: {}", message)),
            Line::from(""),
            Line::from(":set ui-mode standard で基本UIに切り替え"),
            Line::from(""),
            Line::from("Escキーを押して続行してください"),
        ];

        let error_widget = Paragraph::new(error_text)
            .style(Style::default().fg(Color::White))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" 拡張UIエラー ")
                    .border_style(Style::default().fg(Color::Red)),
            )
            .alignment(Alignment::Center);

        f.render_widget(error_widget, area);
    }

    #[allow(dead_code)]
    fn draw_critical_error(f: &mut Frame, _app: &App) {
        let error_text = vec![
            Line::from(""),
            Line::from("拡張UIで深刻なエラーが発生しました"),
            Line::from(""),
            Line::from("アプリケーションを再起動してください"),
            Line::from(""),
            Line::from("一時的な対策: 基本UIモードを使用"),
            Line::from(""),
            Line::from("Ctrl+C で終了"),
        ];

        let error_widget = Paragraph::new(error_text)
            .style(Style::default().fg(Color::White).bg(Color::Red))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" 拡張UI致命的エラー ")
                    .border_style(Style::default().fg(Color::Red)),
            )
            .alignment(Alignment::Center);

        f.render_widget(error_widget, f.size());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::App;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    async fn create_test_app() -> App {
        App::new().await.unwrap()
    }

    #[tokio::test]
    async fn test_enhanced_ui_draw_basic() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = create_test_app().await;

        terminal
            .draw(|f| {
                EnhancedUI::draw(f, &mut app);
            })
            .unwrap();

        // Test passed if no panic occurred
    }

    #[tokio::test]
    async fn test_enhanced_ui_draw_with_help() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = create_test_app().await;
        app.ui_state.toggle_help();

        terminal
            .draw(|f| {
                EnhancedUI::draw(f, &mut app);
            })
            .unwrap();

        assert!(app.show_help());
    }

    #[tokio::test]
    async fn test_enhanced_ui_small_terminal() {
        // Test with very small terminal size
        let backend = TestBackend::new(5, 2);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = create_test_app().await;

        // Should not panic even with tiny size
        terminal
            .draw(|f| {
                EnhancedUI::draw(f, &mut app);
            })
            .unwrap();
    }

    #[test]
    fn test_fallback_error_drawing() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|f| {
                EnhancedUI::draw_fallback_error(f, f.size(), "Test error message");
            })
            .unwrap();
    }

    #[tokio::test]
    async fn test_fallback_title_bar() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let app = create_test_app().await;

        terminal
            .draw(|f| {
                let area = Rect::new(0, 0, 80, 1);
                EnhancedUI::draw_fallback_title_bar(f, &app, area);
            })
            .unwrap();
    }

    #[tokio::test]
    async fn test_fallback_status_bar() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let app = create_test_app().await;

        terminal
            .draw(|f| {
                let area = Rect::new(0, 23, 80, 1);
                EnhancedUI::draw_fallback_status_bar(f, &app, area);
            })
            .unwrap();
    }
}
