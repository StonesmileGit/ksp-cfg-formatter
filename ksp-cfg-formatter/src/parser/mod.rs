use std::fmt::Display;

use pest::iterators::Pair;

pub mod assignment_operator;
pub mod comment;
pub mod document;
pub mod has;
pub mod indices;
pub mod key_val;
pub mod needs;
pub mod node;
pub mod node_item;
pub mod operator;
pub mod pass;
pub mod path;

pub trait ASTPrint {
    #[must_use]
    fn ast_print(
        &self,
        depth: usize,
        indentation: &str,
        line_ending: &str,
        should_collapse: bool,
    ) -> String;
}

/// TODO: Temp
#[derive(Debug, Clone, thiserror::Error)]
pub struct Error {
    pub reason: Reason,
    pub location: Option<Location>,
    pub source_text: String,
}

#[derive(Debug, Clone, Default)]
pub enum Reason {
    Pest(Box<pest::error::Error<Rule>>),
    ParseInt,
    EmptyDocument,
    #[default]
    Unknown,
}

#[derive(Debug, Clone)]
pub struct Location {
    pub start: [usize; 2],
    pub end: [usize; 2],
}

impl From<Pair<'_, Rule>> for Location {
    fn from(rule: Pair<'_, Rule>) -> Self {
        let start = rule.line_col();
        let delta_line = rule
            .as_str()
            .as_bytes()
            .iter()
            .filter(|&&c| c == b'\n')
            .count();
        let last_line = rule.as_str().split('\n').last();
        let col = last_line.map_or(0, |ll| ll.chars().count());
        Location {
            start: [start.0, start.1],
            end: [start.0 + delta_line, col],
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl From<pest::error::Error<Rule>> for Error {
    fn from(value: pest::error::Error<Rule>) -> Self {
        Error {
            reason: Reason::Pest(Box::new(value.clone())),
            location: None,
            source_text: value.to_string(),
        }
    }
}

#[derive(pest_derive::Parser)]
#[grammar = "grammar.pest"]
pub struct Grammar;
