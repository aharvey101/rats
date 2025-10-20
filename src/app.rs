use std::{error::Error, fs, path::PathBuf};
use ratatui::widgets::ListState;
use crate::{config::Config, fuzzy::fuzzy_match, mode::Mode};

pub struct App {
    pub current_path: PathBuf,
    pub items: Vec<PathBuf>,
    pub list_state: ListState,
    pub filter: String,
    pub filtered_items: Vec<(usize, i32)>, // (index, score)
    pub config: Config,
    pub preview_content: Option<String>,
    pub preview_scroll: usize,
    pub mode: Mode,
}

impl App {
    pub fn new(config: Config) -> Result<App, Box<dyn Error>> {
        let current_path = PathBuf::from(&config.directory);
        
        let mut app = App {
            current_path: current_path.clone(),
            items: Vec::new(),
            list_state: ListState::default(),
            filter: config.query.clone(),
            filtered_items: Vec::new(),
            config,
            preview_content: None,
            preview_scroll: 0,
            mode: Mode::Normal,
        };
        app.load_directory()?;
        app.load_preview(); // Load preview for initial selection
        Ok(app)
    }

    pub fn load_directory(&mut self) -> Result<(), Box<dyn Error>> {
        self.items.clear();
        
        // Add parent directory entry if not at root
        if self.current_path.parent().is_some() {
            self.items.push(self.current_path.join(".."));
        }
        
        // Read directory entries
        for entry in fs::read_dir(&self.current_path)? {
            let entry = entry?;
            self.items.push(entry.path());
        }
        
        // Sort: directories first, then files, both alphabetically
        self.items.sort_by(|a, b| {
            // Special case for ".." - always first
            if safe_filename_to_string(a) == ".." {
                return std::cmp::Ordering::Less;
            }
            if safe_filename_to_string(b) == ".." {
                return std::cmp::Ordering::Greater;
            }
            
            match (a.is_dir(), b.is_dir()) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => safe_filename_to_string(a).cmp(&safe_filename_to_string(b)),
            }
        });
        
        self.update_filter();
        Ok(())
    }

    fn update_filter(&mut self) {
        self.filtered_items.clear();
        
        for (i, path) in self.items.iter().enumerate() {
            if let Some(filename) = safe_filename_for_matching(path) {
                if let Some(fuzzy_match) = fuzzy_match(&self.filter, &filename) {
                    self.filtered_items.push((i, fuzzy_match.score));
                }
            }
        }
        
        // Sort by score (higher is better)
        self.filtered_items.sort_by(|a, b| b.1.cmp(&a.1));
        
        // Reset selection to first item
        if self.filtered_items.is_empty() {
            self.list_state.select(None);
        } else {
            self.list_state.select(Some(0));
        }
        self.load_preview();
    }

    pub fn next(&mut self) {
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
        if !self.filtered_items.is_empty() {
            self.list_state.select(Some(i));
            self.load_preview();
        }
    }

    pub fn previous(&mut self) {
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
        if !self.filtered_items.is_empty() {
            self.list_state.select(Some(i));
            self.load_preview();
        }
    }

    pub fn scroll_preview_down(&mut self) {
        if self.preview_content.is_some() {
            self.preview_scroll += 5;
        }
    }

    pub fn scroll_preview_up(&mut self) {
        if self.preview_content.is_some() {
            self.preview_scroll = self.preview_scroll.saturating_sub(5);
        }
    }

    pub fn enter_selected(&mut self) -> Result<Option<PathBuf>, Box<dyn Error>> {
        if let Some(selected) = self.list_state.selected() {
            if let Some(&(item_index, _)) = self.filtered_items.get(selected) {
                if let Some(path) = self.items.get(item_index) {
                    if path.is_dir() {
                        // Navigate to directory
                        if safe_filename_to_string(path) == ".." {
                            // Go to parent directory
                            if let Some(parent) = self.current_path.parent() {
                                self.current_path = parent.to_path_buf();
                            }
                        } else {
                            // Go to subdirectory
                            self.current_path = path.clone();
                        }
                        self.filter.clear(); // Clear filter when navigating
                        self.load_directory()?;
                        return Ok(None);
                    } else {
                        // Return the selected file
                        return Ok(Some(path.clone()));
                    }
                }
            }
        }
        Ok(None)
    }

    pub fn add_char_to_filter(&mut self, c: char) {
        self.filter.push(c);
        self.update_filter();
    }

    pub fn remove_char_from_filter(&mut self) {
        self.filter.pop();
        self.update_filter();
    }

    pub fn clear_filter(&mut self) {
        self.filter.clear();
        self.update_filter();
    }

    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }

    pub fn go_to_top(&mut self) {
        if !self.filtered_items.is_empty() {
            self.list_state.select(Some(0));
            self.load_preview();
        }
    }

    pub fn go_to_bottom(&mut self) {
        if !self.filtered_items.is_empty() {
            self.list_state.select(Some(self.filtered_items.len() - 1));
            self.load_preview();
        }
    }

    pub fn load_preview(&mut self) {
        if let Some(selected) = self.list_state.selected() {
            if let Some(&(item_index, _)) = self.filtered_items.get(selected) {
                if let Some(path) = self.items.get(item_index) {
                    if !path.is_dir() && path.file_name().map_or(false, |name| name != "..") {
                        self.preview_content = self.read_file_content(path);
                        self.preview_scroll = 0;
                    } else {
                        self.preview_content = None;
                        self.preview_scroll = 0;
                    }
                } else {
                    self.preview_content = None;
                    self.preview_scroll = 0;
                }
            } else {
                self.preview_content = None;
                self.preview_scroll = 0;
            }
        } else {
            self.preview_content = None;
            self.preview_scroll = 0;
        }
    }

    fn read_file_content(&self, path: &PathBuf) -> Option<String> {
        // Check if file is likely binary by extension
        if let Some(extension) = path.extension() {
            let ext = extension.to_string_lossy().to_lowercase();
            let binary_extensions = [
                "exe", "bin", "dll", "so", "dylib", "a", "o", "obj",
                "jpg", "jpeg", "png", "gif", "bmp", "ico", "tiff", "webp",
                "mp3", "mp4", "wav", "flac", "ogg", "avi", "mkv", "mov",
                "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx",
                "zip", "tar", "gz", "bz2", "7z", "rar",
            ];
            
            if binary_extensions.contains(&ext.as_str()) {
                return Some(format!("Binary file: {}", path.file_name()?.to_string_lossy()));
            }
        }
        
        // Try to read as text
        match fs::read_to_string(path) {
            Ok(content) => {
                // Check if content looks like binary (contains null bytes)
                if content.contains('\0') {
                    Some(format!("Binary file: {}", path.file_name()?.to_string_lossy()))
                } else {
                    // Limit content size for performance
                    if content.len() > 50000 {
                        Some(format!("{}...\n\n[File truncated - {} bytes total]", 
                                   &content[..50000], content.len()))
                    } else {
                        Some(content)
                    }
                }
            }
            Err(_) => Some("Could not read file".to_string()),
        }
    }
}

pub fn safe_filename_to_string(path: &PathBuf) -> String {
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

pub fn safe_filename_for_matching(path: &PathBuf) -> Option<String> {
    if let Some(name) = path.file_name() {
        Some(name.to_string_lossy().to_string())
    } else {
        None
    }
}