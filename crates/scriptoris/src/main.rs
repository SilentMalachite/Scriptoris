mod app;
mod command_processor;
mod config;
mod editor;
mod enhanced_ui;
mod file_manager;
mod status_manager;
mod ui;
mod ui_state;

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::{io, time::Duration};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logger
    env_logger::init();

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run
    let app = app::App::new().await?;
    let res = run_app(&mut terminal, app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("{err:?}");
    }

    Ok(())
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: app::App) -> Result<()> {
    loop {
        terminal.draw(|f| {
            match app.config.ui_mode {
                config::UIMode::Enhanced => enhanced_ui::EnhancedUI::draw(f, &mut app),
                config::UIMode::Standard => ui::draw(f, &mut app),
            }
        })?;

        // Update status messages (handle auto-expiring messages)
        app.update_status();

        // Check if app should quit
        if app.should_quit() {
            break;
        }

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // Handle Ctrl+C as emergency exit
                if key.code == KeyCode::Char('c') && key.modifiers.contains(event::KeyModifiers::CONTROL) {
                    if app.is_modified() {
                        // Prompt to save before exiting
                        app.ui_state.set_warning_message("Save changes before exit? (y/n/c): ".to_string());
                        app.set_mode(app::Mode::SavePrompt);
                    } else {
                        break;
                    }
                } else if key.code == KeyCode::Char('x') && key.modifiers.contains(event::KeyModifiers::CONTROL) {
                    // Ctrl+X for nano-like users - redirect to vim :q
                    if app.is_modified() {
                        app.ui_state.set_info_message("Save changes? (:wq to save and quit, :q! to quit without saving)".to_string());
                    } else {
                        app.quit();
                    }
                } else {
                    app.handle_key_event(key).await?;
                }
            }
        }
    }
    Ok(())
}
