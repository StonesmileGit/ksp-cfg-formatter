use std::{convert::Infallible, fmt::Display};

use pest::iterators::Pair;

use crate::reader::Rule;

use super::ASTPrint;

#[derive(Debug, Clone, Default)]
pub struct Comment {
    pub text: String,
}

impl<'a> TryFrom<Pair<'a, Rule>> for Comment {
    type Error = Infallible;

    fn try_from(rule: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        Ok(Comment {
            text: rule.as_str().to_string(),
        })
    }
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
