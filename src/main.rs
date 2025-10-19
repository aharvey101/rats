use clap::{Arg, Command};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
use serde::Serialize;
use std::{
    error::Error,
    fs,
    io,
    path::PathBuf,
};

#[derive(Debug, Clone)]
struct Config {
    directory: String,
    query: String,
    json_mode: bool,
}

impl Config {
    fn from_args() -> Config {
        let args: Vec<String> = std::env::args().collect();
        let mut json_mode = false;
        let mut query = String::new();
        let mut directory = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        
        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "--json" => json_mode = true,
                "--query" => {
                    if i + 1 < args.len() {
                        query = args[i + 1].clone();
                        i += 1;
                    }
                }
                path if !path.starts_with("--") => {
                    directory = PathBuf::from(path);
                }
                _ => {}
            }
            i += 1;
        }
        
        Config {
            json_mode,
            query,
            directory: directory.to_string_lossy().to_string(),
        }
    }
}

#[derive(Serialize)]
struct SearchResult {
    path: String,
    score: i32,
    name: String,
    is_dir: bool,
}

fn safe_filename_to_string(path: &PathBuf) -> String {
    if let Some(name) = path.file_name() {
        if let Some(name_str) = name.to_str() {
            // Valid UTF-8
            name_str.to_string()
        } else {
            // Invalid UTF-8 - use lossy conversion
            name.to_string_lossy().to_string()
        }
    } else {
        // Special case: if the path ends with "..", return ".."
        if path.to_string_lossy().ends_with("..") {
            "..".to_string()
        } else {
            // No filename (shouldn't happen for our use case)
            "Unknown".to_string()
        }
    }
}

fn safe_filename_for_matching(path: &PathBuf) -> Option<String> {
    if let Some(name) = path.file_name() {
        Some(name.to_string_lossy().to_string())
    } else {
        None
    }
}

struct App {
    current_path: PathBuf,
    items: Vec<PathBuf>,
    list_state: ListState,
    filter: String,
    filtered_items: Vec<(usize, i32)>, // (index, score)
    config: Config,
}

#[derive(Debug, Clone)]
struct FuzzyMatch {
    score: i32,
    matched_indices: Vec<usize>,
}

fn fuzzy_match(pattern: &str, text: &str) -> Option<FuzzyMatch> {
    if pattern.is_empty() {
        return Some(FuzzyMatch {
            score: 0,
            matched_indices: Vec::new(),
        });
    }

    let pattern = pattern.to_lowercase();
    let text = text.to_lowercase();
    let pattern_chars: Vec<char> = pattern.chars().collect();
    let text_chars: Vec<char> = text.chars().collect();
    
    let mut score = 0i32;
    let mut matched_indices = Vec::new();
    let mut pattern_idx = 0;
    let mut last_match_idx = None;
    
    for (text_idx, &text_char) in text_chars.iter().enumerate() {
        if pattern_idx < pattern_chars.len() && text_char == pattern_chars[pattern_idx] {
            matched_indices.push(text_idx);
            
            // Base score for each match
            score += 10;
            
            // Bonus for consecutive matches
            if let Some(last_idx) = last_match_idx {
                if text_idx == last_idx + 1 {
                    score += 5;
                }
            }
            
            // Bonus for matches at the beginning
            if text_idx == 0 {
                score += 15;
            }
            
            // Bonus for matches after separators
            if text_idx > 0 && (text_chars[text_idx - 1] == '/' || text_chars[text_idx - 1] == '_' || text_chars[text_idx - 1] == '-' || text_chars[text_idx - 1] == '.') {
                score += 10;
            }
            
            last_match_idx = Some(text_idx);
            pattern_idx += 1;
        }
    }
    
    // All pattern characters must be matched
    if pattern_idx == pattern_chars.len() {
        // Penalty for longer text (prefer shorter matches)
        score -= text_chars.len() as i32;
        
        Some(FuzzyMatch {
            score,
            matched_indices,
        })
    } else {
        None
    }
}

impl App {
    fn new(config: Config) -> Result<App, Box<dyn Error>> {
        let current_path = PathBuf::from(&config.directory);
        
        let mut app = App {
            current_path: current_path.clone(),
            items: Vec::new(),
            list_state: ListState::default(),
            filter: config.query.clone(),
            filtered_items: Vec::new(),
            config,
        };
        app.load_directory()?;
        Ok(app)
    }

    fn load_directory(&mut self) -> Result<(), Box<dyn Error>> {
        self.items.clear();
        
        // Add parent directory entry if not at root
        if self.current_path.parent().is_some() {
            self.items.push(self.current_path.join(".."));
        }
        
        let entries = fs::read_dir(&self.current_path)?;
        let mut dirs = Vec::new();
        let mut files = Vec::new();
        
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                dirs.push(path);
            } else {
                files.push(path);
            }
        }
        
        // Sort directories and files separately
        dirs.sort();
        files.sort();
        
        // Add directories first, then files
        self.items.extend(dirs);
        self.items.extend(files);
        
        self.update_filter();
        Ok(())
    }

    fn update_filter(&mut self) {
        self.filtered_items.clear();
        
        if self.filter.is_empty() {
            self.filtered_items = (0..self.items.len()).map(|i| (i, 0)).collect();
        } else {
            let mut matches = Vec::new();
            for (i, item) in self.items.iter().enumerate() {
                if let Some(name_str) = safe_filename_for_matching(item) {
                    // Special handling for parent directory
                    if name_str == ".." {
                        if "..".contains(&self.filter) {
                            matches.push((i, 100)); // High score for parent directory
                        }
                    } else if let Some(fuzzy_match) = fuzzy_match(&self.filter, &name_str) {
                        matches.push((i, fuzzy_match.score));
                    }
                }
            }
            
            // Sort by score (highest first)
            matches.sort_by(|a, b| b.1.cmp(&a.1));
            self.filtered_items = matches;
        }
        
        // Reset selection to first item
        if !self.filtered_items.is_empty() {
            self.list_state.select(Some(0));
        } else {
            self.list_state.select(None);
        }
    }

    fn next(&mut self) {
        if self.filtered_items.is_empty() {
            return;
        }
        
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.filtered_items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn previous(&mut self) {
        if self.filtered_items.is_empty() {
            return;
        }
        
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.filtered_items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn enter_selected(&mut self) -> Result<Option<PathBuf>, Box<dyn Error>> {
        if let Some(selected) = self.list_state.selected() {
            if let Some(&(item_index, _)) = self.filtered_items.get(selected) {
                if let Some(path) = self.items.get(item_index) {
                    if path.is_dir() {
                        self.current_path = path.canonicalize()?;
                        self.load_directory()?;
                        return Ok(None);
                    } else {
                        // Return the file path for opening
                        return Ok(Some(path.clone()));
                    }
                }
            }
        }
        Ok(None)
    }

    fn get_search_results(&self) -> Vec<SearchResult> {
        let mut results = Vec::new();
        let max_results = 100;
        
        // If query is empty or just whitespace, show all files
        let effective_query = self.filter.trim();
        let items_to_process = if effective_query.is_empty() {
            // Show all items when no query
            self.items.iter().enumerate().map(|(i, _)| (i, 0)).collect::<Vec<_>>()
        } else {
            self.filtered_items.clone()
        };
        
        for &(i, score) in items_to_process.iter().take(max_results) {
            let path = &self.items[i];
            let name = safe_filename_to_string(path);
            
            results.push(SearchResult {
                path: path.to_string_lossy().to_string(),
                score,
                name,
                is_dir: path.is_dir(),
            });
        }
        
        results
    }

    fn add_char_to_filter(&mut self, c: char) {
        self.filter.push(c);
        self.update_filter();
    }

    fn remove_char_from_filter(&mut self) {
        self.filter.pop();
        self.update_filter();
    }

    fn clear_filter(&mut self) {
        self.filter.clear();
        self.update_filter();
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let config = Config::from_args();
    
    // If in JSON mode, run in headless mode and output JSON
    if config.json_mode {
        let mut app = App::new(config)?;
        let results = app.get_search_results();
        println!("{}", serde_json::to_string(&results)?);
        return Ok(());
    }
    
    // Setup terminal for interactive mode
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run it
    let mut app = App::new(config)?;
    let res = run_app(&mut terminal, &mut app);

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

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<Option<PathBuf>> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') => return Ok(None),
                    KeyCode::Down | KeyCode::Char('j') => app.next(),
                    KeyCode::Up | KeyCode::Char('k') => app.previous(),
                    KeyCode::Enter => {
                        match app.enter_selected() {
                            Ok(Some(path)) => return Ok(Some(path)),
                            Ok(None) => {}, // Directory navigation, continue
                            Err(_) => {}, // Handle error if needed
                        }
                    }
                    KeyCode::Esc => app.clear_filter(),
                    KeyCode::Backspace => app.remove_char_from_filter(),
                    KeyCode::Char(c) => app.add_char_to_filter(c),
                    _ => {}
                }
            }
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(f.area());

    // Header with current path
    let header = Paragraph::new(format!("Path: {}", app.current_path.display()))
        .block(Block::default().title("Folder Browser").borders(Borders::ALL))
        .style(Style::default().fg(Color::Cyan));
    f.render_widget(header, chunks[0]);

    // File list
    let items: Vec<ListItem> = app
        .filtered_items
        .iter()
        .map(|&(i, _score)| {
            let path = &app.items[i];
            let name = safe_filename_to_string(path);
            
            let display_name = if name == ".." {
                "📁 ..".to_string()
            } else if path.is_dir() {
                format!("📁 {}", name)
            } else {
                format!("📄 {}", name)
            };
            
            ListItem::new(Line::from(Span::raw(display_name)))
        })
        .collect();

    let items_list = List::new(items)
        .block(Block::default().title("Files").borders(Borders::ALL))
        .highlight_style(Style::default().bg(Color::LightBlue).fg(Color::Black))
        .highlight_symbol(">> ");
    
    f.render_stateful_widget(items_list, chunks[1], &mut app.list_state);

    // Footer with filter and help
    let footer_text = if app.filter.is_empty() {
        "Filter: <empty> | j/k or ↓/↑: navigate | Enter: open | q: quit | Esc: clear filter".to_string()
    } else {
        format!("Filter: {} | j/k or ↓/↑: navigate | Enter: open | q: quit | Esc: clear filter", app.filter)
    };
    
    let footer = Paragraph::new(footer_text)
        .block(Block::default().title("Help").borders(Borders::ALL))
        .style(Style::default().fg(Color::Yellow));
    f.render_widget(footer, chunks[2]);
}
