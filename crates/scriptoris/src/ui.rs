//! 標準 UI レンダラー。`WindowManager` の構成に従ってバッファを描画します。

use crate::app::{WindowPane, WindowSplitKind};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::{App, Mode};

fn parse_color(value: &str) -> Option<Color> {
    let hex = value.trim().trim_start_matches('#');
    if hex.len() != 6 {
        log::warn!("Invalid color format: '{}', expected 6 hex digits", value);
        return None;
    }

    // Use catch_unwind for color parsing safety
    match std::panic::catch_unwind(|| {
        let r = u8::from_str_radix(&hex[0..2], 16);
        let g = u8::from_str_radix(&hex[2..4], 16);
        let b = u8::from_str_radix(&hex[4..6], 16);

        match (r, g, b) {
            (Ok(r), Ok(g), Ok(b)) => Some(Color::Rgb(r, g, b)),
            _ => {
                log::warn!("Failed to parse hex color: '{}'", value);
                None
            }
        }
    }) {
        Ok(result) => result,
        Err(e) => {
            log::error!("Panic during color parsing for '{}': {:?}", value, e);
            None
        }
    }
}

pub fn draw(f: &mut Frame, app: &mut App) {
    // Validate frame and app state
    if f.size().width == 0 || f.size().height == 0 {
        log::error!("Invalid frame size: {:?}", f.size());
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Title bar
            Constraint::Min(0),    // Editor area
            Constraint::Length(2), // Status bar
        ])
        .split(f.size());

    // Validate chunk sizes
    if chunks.len() != 3 {
        log::error!("Expected 3 chunks, got {}", chunks.len());
        return;
    }

    // Draw components
    draw_title_bar(f, app, chunks[0]);

    if app.show_help() {
        draw_help(f, chunks[1]);
    } else {
        draw_editor_panes(f, app, chunks[1]);
    }

    draw_status_bar(f, app, chunks[2]);
}

fn draw_editor_panes(f: &mut Frame, app: &mut App, area: Rect) {
    let panes = app.window_manager.panes().to_vec();
    let split_kind = app.window_manager.split_kind();

    match split_kind {
        WindowSplitKind::None | WindowSplitKind::Horizontal | WindowSplitKind::Vertical
            if panes.len() <= 1 =>
        {
            if let Some(pane) = panes.first() {
                draw_single_pane(f, app, area, pane, true);
            }
        }
        WindowSplitKind::Horizontal => {
            let chunk_constraints = if panes.len() > 1 {
                let share = (100 / panes.len().max(1)) as u16;
                vec![Constraint::Percentage(share); panes.len()]
            } else {
                vec![Constraint::Percentage(100)]
            };

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(chunk_constraints)
                .split(area);

            for (idx, pane) in panes.iter().enumerate() {
                let is_current = pane.id == app.window_manager.current_window_id;
                let target_area = chunks.get(idx).copied().unwrap_or_else(|| chunks[0]);
                draw_single_pane(f, app, target_area, pane, is_current);
            }
        }
        WindowSplitKind::Vertical => {
            let chunk_constraints = if panes.len() > 1 {
                let share = (100 / panes.len().max(1)) as u16;
                vec![Constraint::Percentage(share); panes.len()]
            } else {
                vec![Constraint::Percentage(100)]
            };

            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(chunk_constraints)
                .split(area);

            for (idx, pane) in panes.iter().enumerate() {
                let is_current = pane.id == app.window_manager.current_window_id;
                let target_area = chunks.get(idx).copied().unwrap_or_else(|| chunks[0]);
                draw_single_pane(f, app, target_area, pane, is_current);
            }
        }
        WindowSplitKind::None => {
            if let Some(pane) = panes.first() {
                let is_current = pane.id == app.window_manager.current_window_id;
                draw_single_pane(f, app, area, pane, is_current);
            }
        }
    }
}

fn draw_single_pane(f: &mut Frame, app: &mut App, area: Rect, pane: &WindowPane, is_current: bool) {
    if let Some(buffer_index) = app.buffer_manager.find_index_by_id(pane.buffer_id) {
        draw_buffer_by_index(f, app, buffer_index, area, is_current);
    }
}

fn draw_buffer_by_index(
    f: &mut Frame,
    app: &mut App,
    buffer_index: usize,
    area: Rect,
    is_current: bool,
) {
    let (buffer_title, filename, viewport_lines, viewport_offset, cursor_line, cursor_col) = {
        let buffer = &app.buffer_manager.buffers[buffer_index];
        let title = buffer
            .file_path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("[No Name]")
            .to_string();
        let filename = buffer
            .file_path
            .as_ref()
            .and_then(|p| p.to_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "text.md".to_string());
        let viewport_lines = buffer.content.get_viewport_lines();
        let viewport_offset = buffer.content.get_viewport_offset();
        let (cursor_line, cursor_col) = buffer.content.cursor_position();
        (
            title,
            filename,
            viewport_lines,
            viewport_offset,
            cursor_line,
            cursor_col,
        )
    };

    let theme = &app.config.theme;
    let accent_color = theme
        .accent_color
        .as_deref()
        .and_then(parse_color)
        .unwrap_or(Color::Yellow);
    let inactive_border = theme
        .editor_foreground
        .as_deref()
        .and_then(parse_color)
        .unwrap_or(Color::DarkGray);
    let editor_fg = theme
        .editor_foreground
        .as_deref()
        .and_then(parse_color)
        .unwrap_or(Color::White);
    let editor_bg = theme.editor_background.as_deref().and_then(parse_color);

    let border_style = if is_current {
        Style::default().fg(accent_color)
    } else {
        Style::default().fg(inactive_border)
    };

    let mut block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(buffer_title);
    if let Some(bg) = editor_bg {
        block = block.style(Style::default().bg(bg));
    }

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Split for line numbers and content
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(5), // Line numbers
            Constraint::Min(0),    // Content
        ])
        .split(inner);

    // Draw line numbers
    let line_numbers: Vec<String> = (0..viewport_lines.len())
        .map(|i| format!("{:4}", viewport_offset + i + 1))
        .collect();

    let line_numbers_widget =
        Paragraph::new(line_numbers.join("\n")).style(Style::default().fg(inactive_border));

    f.render_widget(line_numbers_widget, chunks[0]);

    let highlighter = app.get_highlighter();
    let syntax = highlighter.find_syntax_for_filename(&filename);
    let content_lines = highlighter.highlight_lines_to_ratatui(&viewport_lines, syntax);
    let content_widget = Paragraph::new(content_lines).style(Style::default().fg(editor_fg));
    f.render_widget(content_widget, chunks[1]);

    // Draw cursor if this is the current window
    if is_current {
        if cursor_line >= viewport_offset && cursor_line < viewport_offset + viewport_lines.len() {
            let screen_line = cursor_line - viewport_offset;
            let line_text = viewport_lines.get(screen_line).cloned().unwrap_or_default();

            // Use accurate text width calculation for cross-platform compatibility
            let logical_prefix: String = line_text.chars().take(cursor_col).collect();
            let display_col: usize = app.text_calculator.str_width(&logical_prefix);

            let x = chunks[1].x + display_col as u16;
            let y = chunks[1].y + screen_line as u16;

            if x < chunks[1].x + chunks[1].width && y < chunks[1].y + chunks[1].height {
                f.set_cursor(x, y);
            }
        }
    }
}

fn draw_title_bar(f: &mut Frame, app: &App, area: Rect) {
    let title = match app.file_path() {
        Some(path) => format!("  Scriptoris -- {}", path.display()),
        None => String::from("  Scriptoris -- [新規ファイル]"),
    };

    let modified_str = if app.is_modified() {
        " [変更あり]"
    } else {
        ""
    };
    let title = format!("{}{}", title, modified_str);

    let status_bg = app
        .config
        .theme
        .status_background
        .as_deref()
        .and_then(parse_color)
        .unwrap_or(Color::Blue);
    let status_fg = app
        .config
        .theme
        .editor_foreground
        .as_deref()
        .and_then(parse_color)
        .unwrap_or(Color::White);

    let title_bar = Paragraph::new(title)
        .style(Style::default().bg(status_bg).fg(status_fg))
        .alignment(Alignment::Left);

    f.render_widget(title_bar, area);
}

fn draw_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Command shortcuts
            Constraint::Length(1), // Status message
        ])
        .split(area);

    let theme = &app.config.theme;
    let status_bg = theme
        .status_background
        .as_deref()
        .and_then(parse_color)
        .unwrap_or(Color::DarkGray);
    let status_fg = theme
        .editor_foreground
        .as_deref()
        .and_then(parse_color)
        .unwrap_or(Color::White);
    let accent = theme
        .accent_color
        .as_deref()
        .and_then(parse_color)
        .unwrap_or(Color::Yellow);

    // Draw command shortcuts or command input
    match app.mode() {
        Mode::Command => {
            let input = Paragraph::new(format!("{}{}", app.status_message(), app.command_buffer()))
                .style(Style::default().fg(accent).bg(status_bg));
            f.render_widget(input, chunks[0]);
        }
        _ => {
            let shortcuts = vec![
                Span::styled(
                    ":",
                    Style::default().fg(accent).add_modifier(Modifier::BOLD),
                ),
                Span::styled(" コマンド  ", Style::default().fg(status_fg)),
                Span::styled(
                    "i",
                    Style::default().fg(accent).add_modifier(Modifier::BOLD),
                ),
                Span::styled(" 挿入  ", Style::default().fg(status_fg)),
                Span::styled(
                    "/",
                    Style::default().fg(accent).add_modifier(Modifier::BOLD),
                ),
                Span::styled(" 検索  ", Style::default().fg(status_fg)),
                Span::styled(
                    "?",
                    Style::default().fg(accent).add_modifier(Modifier::BOLD),
                ),
                Span::styled(" ヘルプ  ", Style::default().fg(status_fg)),
                Span::styled(
                    "hjkl",
                    Style::default().fg(accent).add_modifier(Modifier::BOLD),
                ),
                Span::styled(" Move", Style::default().fg(status_fg)),
            ];

            let shortcut_bar = Paragraph::new(Line::from(shortcuts))
                .style(Style::default().bg(status_bg).fg(status_fg));
            f.render_widget(shortcut_bar, chunks[0]);
        }
    }

    // Draw status message
    let status = Paragraph::new(app.status_message().to_string())
        .style(Style::default().fg(accent).bg(status_bg));
    f.render_widget(status, chunks[1]);
}

fn draw_help(f: &mut Frame, area: Rect) {
    let help_text = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            " ヘルプ — Vim風キー割り当て",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(" モード操作:"),
        Line::from("  i       挿入モード      - 文字入力を開始"),
        Line::from("  Esc     ノーマルモード  - コマンド待機へ戻る"),
        Line::from("  :       コマンドモード  - Vim コマンド入力"),
        Line::from(""),
        Line::from(" 移動(ノーマル):"),
        Line::from("  h j k l                - 左/下/上/右"),
        Line::from("  矢印キー               - 併用可"),
        Line::from(""),
        Line::from(" 編集(ノーマル):"),
        Line::from("  i       挿入           - カーソル前に挿入"),
        Line::from("  a       追加           - カーソル後に挿入"),
        Line::from("  o       改行(下)       - 下に新しい行"),
        Line::from("  O       改行(上)       - 上に新しい行"),
        Line::from("  x       1文字削除       - カーソル位置の文字"),
        Line::from(""),
        Line::from(" ファイル操作:"),
        Line::from("  :w      保存           - ファイルを保存"),
        Line::from("  :q      終了           - 変更なし時のみ終了"),
        Line::from("  :q!     強制終了       - 保存せず終了"),
        Line::from("  :wq     保存して終了   - 保存後に終了"),
        Line::from("  :e file 開く           - 指定ファイルを開く"),
        Line::from(""),
        Line::from(" 検索:"),
        Line::from("  /text   検索           - テキストを検索"),
        Line::from(""),
        Line::from(vec![Span::styled(
            " ? キーでヘルプを閉じる",
            Style::default().add_modifier(Modifier::ITALIC),
        )]),
    ];

    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" ヘルプ ")
                .border_style(Style::default().fg(Color::Blue)),
        )
        .alignment(Alignment::Left);

    f.render_widget(help, area);
}
