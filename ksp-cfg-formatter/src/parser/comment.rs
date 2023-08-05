use super::{ASTPrint, Rule};
use pest::iterators::Pair;
use std::{convert::Infallible, fmt::Display};

/// A comment in the file. Includes the leading `//`
#[derive(Debug, Clone, Copy)]
pub struct Comment<'a> {
    /// Text of the comment, including the leading `//`
    pub text: &'a str,
}

impl<'a> TryFrom<Pair<'a, Rule>> for Comment<'a> {
    type Error = Infallible;

    fn try_from(rule: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        Ok(Comment {
            text: rule.as_str(),
        })
    }
}

impl<'a> Display for Comment<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text)
    }
}

impl<'a> ASTPrint for Comment<'a> {
    fn ast_print(&self, depth: usize, indentation: &str, line_ending: &str, _: bool) -> String {
        let indentation = indentation.repeat(depth);
        format!("{}{}{}", indentation, self.text, line_ending)
    }
}
