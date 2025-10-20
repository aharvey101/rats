use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use crate::{app::{safe_filename_to_string, App}, mode::Mode};

pub fn ui(f: &mut Frame, app: &mut App) {
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

    // Split main area horizontally: file list on left, preview on right
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(chunks[1]);

    // File list (left side)
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
    
    f.render_stateful_widget(items_list, main_chunks[0], &mut app.list_state);

    // File preview (right side)
    let preview_content = if let Some(ref content) = app.preview_content {
        let lines: Vec<&str> = content.lines().collect();
        let start_line = app.preview_scroll;
        let visible_height = main_chunks[1].height.saturating_sub(2) as usize; // Account for borders
        
        let visible_lines = if start_line < lines.len() {
            let end_line = std::cmp::min(start_line + visible_height, lines.len());
            lines[start_line..end_line].join("\n")
        } else {
            String::new()
        };
        
        // Show scroll indicators
        let scroll_info = if lines.len() > visible_height {
            format!(" [{}..{}/{}]", start_line + 1, 
                   std::cmp::min(start_line + visible_height, lines.len()), 
                   lines.len())
        } else {
            String::new()
        };
        
        (visible_lines, format!("Preview{}", scroll_info))
    } else {
        ("Select a file to preview".to_string(), "Preview".to_string())
    };

    let preview_widget = Paragraph::new(preview_content.0)
        .block(Block::default().title(preview_content.1).borders(Borders::ALL))
        .style(Style::default().fg(Color::White));
    
    f.render_widget(preview_widget, main_chunks[1]);

    // Footer with filter and help
    let mode_indicator = match app.mode {
        Mode::Normal => "NORMAL",
        Mode::Insert => "INSERT",
    };
    
    let help_text = match app.mode {
        Mode::Normal => "j/k: navigate | h/l: scroll preview | Enter: open | i/: insert mode | gg/G: top/bottom | q: quit | Esc: clear filter",
        Mode::Insert => "Type to filter | Enter: open | Esc: normal mode | Backspace: delete char",
    };
    
    let footer_text = if app.filter.is_empty() {
        format!("-- {} -- | Filter: <empty> | {}", mode_indicator, help_text)
    } else {
        format!("-- {} -- | Filter: {} | {}", mode_indicator, app.filter, help_text)
    };
    
    let footer_color = match app.mode {
        Mode::Normal => Color::Cyan,
        Mode::Insert => Color::Green,
    };
    
    let footer = Paragraph::new(footer_text)
        .block(Block::default().title("Help").borders(Borders::ALL))
        .style(Style::default().fg(footer_color));
    f.render_widget(footer, chunks[2]);
}