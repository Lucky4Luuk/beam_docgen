use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::ir::*;

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

fn get_all_files(game_dir: &str, extensions: &[&str]) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_files(Path::new(game_dir), extensions, &mut files);
    files
}

fn collect_files(dir: &Path, extensions: &[&str], files: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_dir() {
            collect_files(&path, extensions, files);
        } else if path.is_file()
            && path
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| {
                    extensions
                        .iter()
                        .any(|allowed| ext.eq_ignore_ascii_case(allowed))
                })
        {
            files.push(path);
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct CodeFile {
    pub path: String,
    pub content: Vec<String>,

    pub func_calls: Vec<(String, usize)>,
}

impl CodeFile {
    /// This function extracts every single function call and corresponding line number
    /// from the file.
    fn extract_all_lua_func_calls(&mut self) {
        let mut results = Vec::new();

        for (line_idx, line) in self.content.iter().enumerate() {
            let bytes = line.as_bytes();
            let mut i = 0;

            while i < bytes.len() {
                let c = bytes[i] as char;

                if c.is_ascii_alphabetic() || c == '_' {
                    let start = i;
                    i += 1;

                    while i < bytes.len() {
                        let ch = bytes[i] as char;
                        if ch.is_ascii_alphanumeric() || ch == '_' {
                            i += 1;
                        } else {
                            break;
                        }
                    }

                    let ident = &line[start..i];
                    let after = &line[i..];
                    let trimmed_after = after.trim_start();

                    if ident != "function" && trimmed_after.starts_with('(') {
                        results.push((ident.to_string(), line_idx + 1));
                    }
                } else {
                    i += 1;
                }
            }
        }

        self.func_calls = results;
    }

    fn look_for_func_call(&self, func_name: &str) -> Vec<usize> {
        self.func_calls
            .iter()
            .filter_map(
                |(name, line)| {
                    if name == func_name { Some(*line) } else { None }
                },
            )
            .collect()
    }

    pub fn get_line(&self, line: usize) -> Option<&String> {
        self.content.get(line)
    }

    pub fn get_section(&self, start: usize, end: usize) -> Vec<String> {
        let start_idx = start.saturating_sub(1);
        let end_idx = end.min(self.content.len());

        if start_idx >= end_idx {
            return Vec::new();
        }

        self.content[start_idx..end_idx].to_vec()
    }

    /// Specifically searches from line down to find the actual definition starting with `function`
    /// Searches down a maximum of 5 lines. Specifically only finds lines
    /// starting with `function`, because we don't really need this for
    /// any functions that are defined in other ways
    pub fn get_func_def(&self, line: usize) -> Option<String> {
        for i in line..(line + 5) {
            if let Some(s) = self.get_line(i) {
                let s = s.trim_start();
                if s.starts_with("function") {
                    return Some(s.to_string());
                } else if s.starts_with("local function") {
                    return Some(s[6..].to_string());
                }
            }
        }
        None
    }

    pub fn get_func_decl_with_comments(&self, start: usize, end: usize) -> (Vec<String>, usize) {
        let start_idx = start.saturating_sub(1);
        let end_idx = end.min(self.content.len());

        if start_idx >= end_idx {
            return (Vec::new(), 0);
        }

        let mut actual_start = start_idx;
        let mut offset = 0;

        while actual_start > 0 {
            let prev_line = &self.content[actual_start - 1];
            let trimmed = prev_line.trim();

            if trimmed.starts_with("--") || trimmed.is_empty() {
                actual_start -= 1;
                offset += 1;
            } else {
                break;
            }
        }

        (self.content[actual_start..end_idx].to_vec(), offset)
    }

    pub fn get_func_call_lines(&self, lines: &[usize]) -> Vec<FuncCaller> {
        lines
            .iter()
            .map(|line| {
                let surrounding_code = self
                    .content
                    .iter()
                    .enumerate()
                    .filter_map(|(idx, content_line)| {
                        let current_line = idx + 1;
                        if current_line >= line.saturating_sub(1) && current_line <= line + 1 {
                            Some(content_line.to_string())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();
                FuncCaller {
                    line: *line,
                    surrounding_code,
                }
            })
            .collect()
    }
}

fn load_all_files(game_dir: &str, file_paths: Vec<PathBuf>) -> Vec<CodeFile> {
    file_paths
        .into_par_iter()
        .filter_map(|path| {
            let content = fs::read_to_string(&path).ok()?;
            let path = PathBuf::from(path.strip_prefix(game_dir).unwrap_or(&path).to_path_buf());
            let path = path.to_string_lossy().replace('\\', "/");
            Some(CodeFile {
                path,
                content: content.lines().map(|s| s.to_string()).collect(),
                func_calls: Vec::new(),
            })
        })
        .collect()
}

pub fn look_up_src_info(root: &mut Table, game_dir: &str) -> HashMap<String, CodeFile> {
    // First we must extract every function name and full name
    let look_ups = root.get_all_function_names();
    println!("Functions to look up: {}", look_ups.len());

    // Secondly, we must extract a list of every single lua/js file
    // in the game directory.
    let file_paths = get_all_files(game_dir, &["lua", "js"]);

    // Now we can simply load every single one of these files into
    // memory, as it's just code and won't take up that much RAM.
    let mut files = load_all_files(game_dir, file_paths);
    println!("Files loaded: {}", files.len());

    // Time to extract all func calls!
    println!("Extracting all function calls from each file...");
    files
        .par_iter_mut()
        .for_each(|f| f.extract_all_lua_func_calls());
    println!("Done!");

    // Now that we have all files loaded into memory, we can
    // search them in parallel!
    println!("Starting caller search...");
    let all_info: Vec<_> = look_ups
        .par_iter()
        .map(|(full_name, name)| {
            let mut all_calls = HashMap::new();
            for file in &files {
                let calls = file.look_for_func_call(name);
                if calls.len() > 0 {
                    let path = file.path.clone();
                    all_calls.insert(path, calls);
                }
            }

            (full_name, all_calls)
        })
        .collect();
    println!("");
    println!("Looked up every single call!");

    // We now have every single function call registered, tied to the full name
    // All we have to do now is put the data in!
    for (full_name, calls) in all_info {
        if let Some(f) = root.get_function_mut_by_full_name(full_name) {
            f.set_calls(calls);
        } else {
            panic!("Function not found: {}", full_name);
        }
    }
    println!("Function caller look up is done!");

    files
        .into_iter()
        .map(|file| (file.path.clone(), file))
        .collect()
}
