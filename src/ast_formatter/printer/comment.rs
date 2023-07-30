use std::fmt::Display;

use super::ASTPrint;

#[derive(Debug, Clone, Default)]
pub struct Comment {
    pub text: String,
}

impl Display for Comment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text)
    }
}

impl ASTPrint for Comment {
    fn ast_print(&self, depth: usize, indentation: &str, line_ending: &str, _: bool) -> String {
        let indentation = indentation.repeat(depth);
        format!("{}{}{}", indentation, self.text, line_ending)
    }
}
