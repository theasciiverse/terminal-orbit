use std::collections::VecDeque;
use std::fs;
use std::path::{Path, PathBuf};

/// Token stream backed by an include-file stack.
/// Each entry in `stack` is the remaining tokens for one file level.
pub struct Parser {
    stack: Vec<VecDeque<String>>,
    /// Directory where mission and include files live.
    pub missions_dir: PathBuf,
}

impl Parser {
    pub fn new(missions_dir: impl Into<PathBuf>) -> Self {
        Parser { stack: Vec::new(), missions_dir: missions_dir.into() }
    }

    /// Push the tokens from a file onto the stack.
    pub fn push_file(&mut self, path: &Path) -> Result<(), String> {
        let text = fs::read_to_string(path)
            .map_err(|e| format!("Cannot open {}: {}", path.display(), e))?;
        self.stack.push(tokenize(&text));
        Ok(())
    }

    /// Push an include file (looked up relative to missions_dir).
    pub fn push_include(&mut self, filename: &str) -> Result<(), String> {
        let path = self.missions_dir.join(filename);
        self.push_file(&path)
    }

    /// Return the next token, popping exhausted file levels.
    pub fn next(&mut self) -> Option<String> {
        loop {
            let top = self.stack.last_mut()?;
            if let Some(tok) = top.pop_front() {
                return Some(tok);
            }
            self.stack.pop();
        }
    }

    /// Return the next token or an error string.
    pub fn require(&mut self) -> Result<String, String> {
        self.next().ok_or_else(|| "Unexpected end of mission file".to_string())
    }

    /// Expect the next token to be `{`.
    pub fn require_brace(&mut self) -> Result<(), String> {
        let t = self.require()?;
        if t != "{" {
            return Err(format!("Expected '{{', found '{}'", t));
        }
        Ok(())
    }
}

/// Split `text` into whitespace-delimited tokens, stripping `/* ... */` comments.
fn tokenize(text: &str) -> VecDeque<String> {
    let raw: VecDeque<String> = text.split_whitespace().map(str::to_string).collect();
    strip_comments(raw)
}

fn strip_comments(mut input: VecDeque<String>) -> VecDeque<String> {
    let mut out = VecDeque::new();
    let mut in_comment = false;
    while let Some(tok) = input.pop_front() {
        if in_comment {
            if tok == "*/" { in_comment = false; }
        } else if tok == "/*" {
            in_comment = true;
        } else {
            out.push_back(tok);
        }
    }
    out
}
