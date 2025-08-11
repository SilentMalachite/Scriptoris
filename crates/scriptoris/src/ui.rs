use crate::app::{Window, Split, Buffer};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, Mode};

pub fn draw(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // Title bar
            Constraint::Min(0),     // Editor area
            Constraint::Length(2),  // Status bar
        ])
        .split(f.size());

    draw_title_bar(f, app, chunks[0]);
    
    if app.show_help() {
        draw_help(f, chunks[1]);
    } else {
        draw_windows(f, app, chunks[1], &app.window_manager.get_root().clone());
    }
    
    draw_status_bar(f, app, chunks[2]);
}

fn draw_windows(f: &mut Frame, app: &mut App, area: Rect, window: &Window) {
    match &window.split {
        Split::Leaf { buffer_id } => {
            // Draw single buffer in this window
            if let Ok(()) = app.buffer_manager.switch_to_buffer(*buffer_id) {
                let buffer = app.buffer_manager.get_current();
                let is_current = window.id == app.window_manager.current_window_id;
                draw_buffer(f, buffer, area, is_current);
            }
        }
        Split::Horizontal { top, bottom, ratio } => {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage((ratio * 100.0) as u16),
                    Constraint::Percentage(((1.0 - ratio) * 100.0) as u16),
                ])
                .split(area);
            
            draw_windows(f, app, chunks[0], top);
            
            // Draw horizontal separator
            let separator = Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(Color::DarkGray));
            f.render_widget(separator, chunks[1]);
            
            draw_windows(f, app, chunks[1], bottom);
        }
        Split::Vertical { left, right, ratio } => {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage((ratio * 100.0) as u16),
                    Constraint::Percentage(((1.0 - ratio) * 100.0) as u16),
                ])
                .split(area);
            
            draw_windows(f, app, chunks[0], left);
            
            // Draw vertical separator
            let separator = Block::default()
                .borders(Borders::LEFT)
                .border_style(Style::default().fg(Color::DarkGray));
            f.render_widget(separator, chunks[1]);
            
            draw_windows(f, app, chunks[1], right);
        }
    }
}

fn draw_buffer(f: &mut Frame, buffer: &Buffer, area: Rect, is_current: bool) {
    let border_style = if is_current {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(
            buffer.file_path
                .as_ref()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or("[No Name]")
        );
    
    let inner = block.inner(area);
    f.render_widget(block, area);
    
    // Split for line numbers and content
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(5),  // Line numbers
            Constraint::Min(0),     // Content
        ])
        .split(inner);
    
    // Draw line numbers
    let line_count = buffer.content.line_count();
    let viewport_lines = buffer.content.get_viewport_lines();
    let start_line = buffer.content.get_viewport_offset();
    
    let line_numbers: Vec<String> = (0..viewport_lines.len())
        .map(|i| format!("{:4}", start_line + i + 1))
        .collect();
    
    let line_numbers_widget = Paragraph::new(line_numbers.join("\n"))
        .style(Style::default().fg(Color::DarkGray));
    
    f.render_widget(line_numbers_widget, chunks[0]);
    
    // Draw content
    let content = viewport_lines.join("\n");
    let content_widget = Paragraph::new(content)
        .style(Style::default().fg(Color::White));
    
    f.render_widget(content_widget, chunks[1]);
    
    // Draw cursor if this is the current window
    if is_current {
        let (cursor_line, cursor_col) = buffer.content.cursor_position();
        let viewport_offset = buffer.content.get_viewport_offset();
        
        if cursor_line >= viewport_offset && cursor_line < viewport_offset + viewport_lines.len() {
            let screen_line = cursor_line - viewport_offset;
            let x = chunks[1].x + cursor_col as u16;
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
        None => String::from("  Scriptoris -- [New File]"),
    };
    
    let modified_str = if app.is_modified() { " [Modified]" } else { "" };
    let title = format!("{}{}", title, modified_str);
    
    let title_bar = Paragraph::new(title)
        .style(Style::default().bg(Color::Blue).fg(Color::White))
        .alignment(Alignment::Left);
    
    f.render_widget(title_bar, area);
}

fn draw_editor(f: &mut Frame, app: &mut App, area: Rect) {
    let editor_area = if app.config.editor.line_numbers {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(6),  // Line numbers
                Constraint::Min(0),     // Editor content
            ])
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(0),  // No line numbers
                Constraint::Min(0),     // Editor content
            ])
            .split(area)
    };

    // Update viewport height
    app.editor.set_viewport_height(area.height as usize);

    // Draw line numbers if enabled
    if app.config.editor.line_numbers {
        draw_line_numbers(f, app, editor_area[0]);
    }

    // Draw editor content
    let lines = app.editor.get_viewport_lines();
    let (cursor_line, cursor_col) = app.editor.cursor_position();
    
    let mut text_lines = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        let line_str = line.trim_end_matches('\n');
        
        if app.config.editor.highlight_current_line && i == cursor_line {
            text_lines.push(Line::from(vec![
                Span::styled(line_str, Style::default().bg(Color::DarkGray))
            ]));
        } else {
            text_lines.push(Line::from(line_str));
        }
    }
    
    let editor_content = Paragraph::new(text_lines)
        .style(Style::default().fg(Color::White))
        .block(Block::default().borders(Borders::NONE))
        .wrap(Wrap { trim: false });
    
    f.render_widget(editor_content, editor_area[1]);
    
    // Set cursor position
    let cursor_x = editor_area[1].x + cursor_col as u16;
    let cursor_y = editor_area[1].y + cursor_line as u16;
    f.set_cursor(cursor_x, cursor_y);
}

fn draw_line_numbers(f: &mut Frame, app: &App, area: Rect) {
    let lines = app.editor.get_viewport_lines();
    let mut line_numbers = Vec::new();
    
    for i in 0..lines.len() {
        line_numbers.push(Line::from(format!("{:>4} ", i + 1)));
    }
    
    let line_number_widget = Paragraph::new(line_numbers)
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::RIGHT));
    
    f.render_widget(line_number_widget, area);
}

fn draw_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // Command shortcuts
            Constraint::Length(1),  // Status message
        ])
        .split(area);

    // Draw command shortcuts or command input
    match app.mode() {
        Mode::Command => {
            let input = Paragraph::new(format!("{}{}", app.status_message(), app.command_buffer()))
                .style(Style::default().fg(Color::Yellow));
            f.render_widget(input, chunks[0]);
        }
        _ => {
            let shortcuts = vec![
                Span::styled(":", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                Span::raw(" Command  "),
                Span::styled("i", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                Span::raw(" Insert  "),
                Span::styled("/", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                Span::raw(" Search  "),
                Span::styled("?", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                Span::raw(" Help  "),
                Span::styled("hjkl", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                Span::raw(" Move"),
            ];
            
            let shortcut_bar = Paragraph::new(Line::from(shortcuts))
                .style(Style::default().bg(Color::DarkGray));
            f.render_widget(shortcut_bar, chunks[0]);
        }
    }

    // Draw status message
    let status = Paragraph::new(app.status_message().to_string())
        .style(Style::default().fg(Color::Yellow));
    f.render_widget(status, chunks[1]);
}

fn draw_help(f: &mut Frame, area: Rect) {
    let help_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(" HELP -- Vim-style Key Bindings", Style::default().add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(" Mode Commands:"),
        Line::from("  i       Insert mode    - Start inserting text"),
        Line::from("  Esc     Normal mode    - Return to command mode"),
        Line::from("  :       Command mode   - Enter vim commands"),
        Line::from(""),
        Line::from(" Movement (Normal Mode):"),
        Line::from("  h j k l                - Left, Down, Up, Right"),
        Line::from("  Arrow keys             - Also supported"),
        Line::from(""),
        Line::from(" Editing (Normal Mode):"),
        Line::from("  i       Insert         - Insert before cursor"),
        Line::from("  a       Append         - Insert after cursor"),
        Line::from("  o       Open line      - New line below"),
        Line::from("  O       Open line      - New line above"),
        Line::from("  x       Delete char    - Delete character under cursor"),
        Line::from(""),
        Line::from(" File Commands:"),
        Line::from("  :w      Write          - Save file"),
        Line::from("  :q      Quit           - Exit (if no changes)"),
        Line::from("  :q!     Force quit     - Exit without saving"),
        Line::from("  :wq     Write & quit   - Save and exit"),
        Line::from("  :e file Edit           - Open file"),
        Line::from(""),
        Line::from(" Search:"),
        Line::from("  /text   Search         - Search for text"),
        Line::from(""),
        Line::from(vec![
            Span::styled(" Press ? to exit help", Style::default().add_modifier(Modifier::ITALIC)),
        ]),
    ];

    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Help ")
                .border_style(Style::default().fg(Color::Blue)),
        )
        .alignment(Alignment::Left);

    f.render_widget(help, area);
}