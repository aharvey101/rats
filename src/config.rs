use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub directory: String,
    pub query: String,
    pub json_mode: bool,
}

impl Config {
    pub fn from_args() -> Config {
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