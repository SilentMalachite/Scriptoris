mod app;
mod command_processor;
mod config;
mod editor;
mod enhanced_ui;
mod file_manager;
mod highlight;
mod session_manager;
mod status_manager;
mod text_width;
mod ui;
mod ui_state;

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use log::LevelFilter;
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::{env, io, time::Duration};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logger with debug fallback for development
    let mut logger = env_logger::Builder::from_default_env();
    if std::env::var_os("RUST_LOG").is_none() {
        logger.filter_level(LevelFilter::Info);
        logger.filter_module("scriptoris", LevelFilter::Debug);
    }
    logger.init();

    // Setup panic hook to restore terminal
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = restore_terminal();
        original_hook(panic_info);
    }));

    // Setup terminal
    if let Err(e) = enable_raw_mode() {
        eprintln!("ターミナルの初期化に失敗しました: {}", e);
        return Err(e.into());
    }
    let mut stdout = io::stdout();
    if let Err(e) = execute!(stdout, EnterAlternateScreen, EnableMouseCapture) {
        let _ = disable_raw_mode();
        eprintln!("ターミナルの設定に失敗しました: {}", e);
        return Err(e.into());
    }
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    let mut app = match app::App::new().await {
        Ok(app) => {
            log::info!("Application initialized successfully");
            app
        }
        Err(e) => {
            restore_terminal()?;
            eprintln!("アプリケーションの初期化に失敗しました: {}", e);
            if let Some(source) = e.source() {
                eprintln!("詳細: {}", source);
            } else {
                eprintln!("詳細: 不明なエラー");
            }
            return Err(e);
        }
    };

    // Load file from command line if provided
    if args.len() > 1 {
        let file_path = std::path::PathBuf::from(&args[1]);

        // Validate file path arguments
        match app.file_manager.open_file(file_path.clone()).await {
            Ok(content) => {
                let content_str = content.clone();
                app.get_current_editor_mut().set_content(content);

                // Notify LSP plugin of document opening
                #[cfg(feature = "lsp")]
                app.notify_lsp_document_opened(&file_path, &content_str).await;

                app.ui_state.set_info_message(
                    format!("ファイルを読み込みました: {}", args[1])
                );
                log::info!("Successfully loaded file from command line: {}", args[1]);
            }
            Err(e) => {
                let error_msg = format!("ファイル読み込みエラー: {}", e);
                app.ui_state.set_error_message(error_msg.clone());
                log::error!("Failed to load file '{}': {}", args[1], e);

                // If it's a critical error (not file not found), show it prominently
                if !e.to_string().contains("見つかりません") {
                    app.ui_state.set_warning_message(
                        format!("続行しますが、ファイル '{}' は読み込まれませんでした", args[1])
                    );
                }
            }
        }
    } else {
        log::info!("No file specified, starting with empty buffer");
    }

    let res = run_app(&mut terminal, app).await;

    // Restore terminal
    restore_terminal()?;

    if let Err(err) = res {
        eprintln!("アプリケーション実行中にエラーが発生しました: {}", err);

        // Provide user-friendly error messages
        if let Some(source) = err.source() {
            eprintln!("原因: {}", source);
        }

        // Log the error for debugging
        log::error!("Application error: {}", err);

        // Attempt to provide recovery suggestion
        if err.to_string().contains("terminal") {
            eprintln!("提案: ターミナルが互換性モードで実行されているか確認してください");
        } else if err.to_string().contains("permission") {
            eprintln!("提案: ファイルのアクセス権限を確認してください");
        } else if err.to_string().contains("memory") {
            eprintln!("提案: 利用可能なメモリを確保してください");
        } else {
            eprintln!("提案: 問題が続く場合は、バグ報告してください");
        }
    }

    Ok(())
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: app::App) -> Result<()> {
    // Main application loop with safe error handling
    loop {
        // Draw UI
        if let Err(e) = terminal.draw(|f| {
            match app.config.ui_mode {
                config::UIMode::Enhanced => enhanced_ui::EnhancedUI::draw(f, &mut app),
                config::UIMode::Standard => ui::draw(f, &mut app),
            }
        }) {
            log::error!("Terminal draw error: {}", e);
            // Continue running despite draw errors
        }

        // Update status messages
        app.update_status();

        // Check if app should quit
        if app.should_quit() {
            log::info!("Application shutdown requested");
            break;
        }

        // Handle events
        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) => {
                    if let Err(e) = handle_key_event_safe(key, &mut app).await {
                        log::error!("Key event handling error: {}", e);
                        app.ui_state.set_error_message(format!("キー処理エラー: {}", e));
                    }
                }
                Event::Resize(_, _) => {
                    log::info!("Terminal resized");
                    // Handle resize implicitly through next draw
                }
                Event::Mouse(_) => {
                    // Ignore mouse events for now
                }
                _ => {}
            }
        }
    }

    log::info!("Application loop ended successfully");
    Ok(())
}

async fn handle_key_event_safe(key: crossterm::event::KeyEvent, app: &mut app::App) -> Result<()> {
    // Handle Ctrl+C as emergency exit
    if key.code == KeyCode::Char('c')
        && key.modifiers.contains(event::KeyModifiers::CONTROL)
    {
        log::info!("Emergency exit requested via Ctrl+C");
        if app.is_modified() {
            // Prompt to save before exiting
            app.ui_state
                .set_warning_message("Save changes before exit? (y/n/c): ".to_string());
            app.set_mode(app::Mode::SavePrompt);
        } else {
            app.quit();
        }
        return Ok(());
    } else if key.code == KeyCode::Char('x')
        && key.modifiers.contains(event::KeyModifiers::CONTROL)
    {
        log::info!("Nano-style exit requested via Ctrl+X");
        // Ctrl+X for nano-like users - redirect to vim :q
        if app.is_modified() {
            app.ui_state.set_info_message(
                "Save changes? (:wq to save and quit, :q! to quit without saving)"
                    .to_string(),
            );
        } else {
            app.quit();
        }
        return Ok(());
    }

    // Regular key event handling
    app.handle_key_event(key).await
}

/// Restore terminal to normal state
fn restore_terminal() -> Result<()> {
    disable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, LeaveAlternateScreen, DisableMouseCapture)?;
    // Attempt to show cursor, but don't fail if it errors
    let _ = execute!(stdout, crossterm::cursor::Show);
    Ok(())
}
