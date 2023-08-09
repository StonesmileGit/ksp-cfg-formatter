use std::fmt::Display;

use pest::iterators::Pair;

mod assignment_operator;
mod comment;
mod document;
mod has;
mod indices;
mod key_val;
mod needs;
mod node;
mod node_item;
mod operator;
mod pass;
mod path;

pub use assignment_operator::AssignmentOperator;
pub use comment::Comment;
pub use document::Document;
pub use has::{HasBlock, HasPredicate, MatchType};
pub use indices::{ArrayIndex, Index};
pub use key_val::KeyVal;
pub use needs::{ModClause, NeedsBlock, OrClause};
pub use node::Node;
pub use node_item::NodeItem;
pub use operator::Operator;
pub use pass::Pass;
pub use path::{Path, PathSegment, PathStart};

/// Indicates that the type can be pretty-printed as part of the formatter
pub trait ASTPrint {
    /// Pretty-print the type to a string, ready to be written to file/output
    #[must_use]
    fn ast_print(
        &self,
        depth: usize,
        indentation: &str,
        line_ending: &str,
        should_collapse: bool,
    ) -> String;
}

/// Error from the parser, with context
#[derive(Debug, Clone, thiserror::Error)]
pub struct Error {
    /// The reason for the error
    pub reason: Reason,
    /// Optional line/col span indicating the origin of the error
    pub location: Option<Location>,
    /// Source string from which the error occured
    pub source_text: String,
}

/// Reason for the error that occured
#[derive(Debug, Clone, Default)]
pub enum Reason {
    /// An error from the PEST parser
    Pest(Box<pest::error::Error<Rule>>),
    /// Parsing of an int failed
    ParseInt,
    /// Custom error with reason provided
    Custom(String),
    /// Unknown error
    #[default]
    Unknown,
}

/// Location of an error, as a span between `start` and `end`
#[derive(Debug, Clone)]
pub struct Location {
    /// line/col of the start of the error
    pub start: [usize; 2],
    /// line/col of the end of the error
    pub end: [usize; 2],
}

impl Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.end[0] - self.start[0] > 0 {
            write!(
                f,
                "[{}, {}] to [{}, {}]",
                self.start[0], self.start[1], self.end[0], self.end[1]
            )
        } else {
            write!(
                f,
                "Ln {}, Col {}-{}",
                self.start[0], self.start[1], self.end[1]
            )
        }
    }
}

impl From<Pair<'_, Rule>> for Location {
    fn from(rule: Pair<'_, Rule>) -> Self {
        let start = rule.line_col();
        let delta_line = rule.as_str().chars().filter(|&c| c == '\n').count();
        let last_line = rule.as_str().split('\n').last();
        let col = last_line.map_or(0, |ll| ll.chars().count());
        Location {
            start: [start.0, start.1],
            end: [
                start.0 + delta_line,
                if delta_line > 0 { col } else { start.1 + col },
            ],
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.reason {
            Reason::Pest(pest) => write!(f, "{pest}"),
            Reason::ParseInt => todo!(),
            Reason::Custom(text) => write!(
                f,
                "{}, found '{}'{}",
                text,
                self.source_text,
                self.location
                    .clone()
                    .map_or(String::new(), |loc| format!(" at {loc}"))
            ),
            Reason::Unknown => write!(
                f,
                "UNKNOWN ERROR. source: {}{}",
                self.source_text,
                self.location
                    .clone()
                    .map_or(String::new(), |loc| format!(" at {loc}"))
            ),
        }
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

pub use grammar::{Grammar, Rule};
mod grammar {
    #![allow(missing_docs)]
    #[derive(pest_derive::Parser)]
    #[grammar = "grammar.pest"]
    pub struct Grammar;
}
