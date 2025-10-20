use serde::Serialize;

#[derive(Debug, Clone)]
pub struct FuzzyMatch {
    pub score: i32,
    pub matched_indices: Vec<usize>,
}

#[derive(Serialize)]
pub struct SearchResult {
    pub path: String,
    pub score: i32,
    pub name: String,
    pub is_dir: bool,
}

pub fn fuzzy_match(pattern: &str, text: &str) -> Option<FuzzyMatch> {
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