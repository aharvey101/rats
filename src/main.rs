mod app;
mod config;
mod fuzzy;
mod mode;
mod ui;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::{error::Error, io, path::PathBuf};

use app::App;
use config::Config;
use mode::Mode;
use ui::ui;

fn main() -> Result<(), Box<dyn Error>> {
    // Parse configuration
    let config = Config::from_args();

    // Handle JSON mode
    if config.json_mode {
        // JSON mode implementation would go here if needed
        // For now, just print that it's not implemented in this refactor
        eprintln!("JSON mode not implemented in this version");
        return Ok(());
    }

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run it
    let app = App::new(config)?;
    let res = run_app(&mut terminal, app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    match res {
        Ok(Some(selected_file)) => {
            // Print the selected file path for external tools (like Neovim) to capture
            println!("{}", selected_file.display());
        }
        Ok(None) => {
            // User quit without selecting anything
        }
        Err(err) => {
            eprintln!("Error: {:?}", err);
            return Err(err.into());
        }
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<Option<PathBuf>> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match app.mode {
                    Mode::Normal => {
                        match key.code {
                            KeyCode::Char('q') => return Ok(None),
                            KeyCode::Char('i') => app.set_mode(Mode::Insert),
                            KeyCode::Char('/') => app.set_mode(Mode::Insert),
                            KeyCode::Down | KeyCode::Char('j') => app.next(),
                            KeyCode::Up | KeyCode::Char('k') => app.previous(),
                            KeyCode::Left | KeyCode::Char('h') => app.scroll_preview_up(),
                            KeyCode::Right | KeyCode::Char('l') => app.scroll_preview_down(),
                            KeyCode::Char('g') => {
                                // Handle 'gg' - go to top
                                if let Event::Key(next_key) = event::read()? {
                                    if next_key.kind == KeyEventKind::Press {
                                        if let KeyCode::Char('g') = next_key.code {
                                            app.go_to_top();
                                        }
                                    }
                                }
                            },
                            KeyCode::Char('G') => app.go_to_bottom(),
                            KeyCode::Enter => {
                                match app.enter_selected() {
                                    Ok(Some(path)) => return Ok(Some(path)),
                                    Ok(None) => {}, // Directory navigation, continue
                                    Err(_) => {}, // Handle error if needed
                                }
                            }
                            KeyCode::Esc => app.clear_filter(),
                            _ => {}
                        }
                    },
                    Mode::Insert => {
                        match key.code {
                            KeyCode::Esc => app.set_mode(Mode::Normal),
                            KeyCode::Enter => {
                                match app.enter_selected() {
                                    Ok(Some(path)) => return Ok(Some(path)),
                                    Ok(None) => {}, // Directory navigation, continue
                                    Err(_) => {}, // Handle error if needed
                                }
                            }
                            KeyCode::Backspace => app.remove_char_from_filter(),
                            KeyCode::Char(c) => app.add_char_to_filter(c),
                            _ => {}
                        }
                    }
                }
            }
        }
    }
}