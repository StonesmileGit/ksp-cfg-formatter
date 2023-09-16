use std::{fmt::Display, num::TryFromIntError};

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

/// Module with the same parser implemented using nom
pub mod nom;

pub use assignment_operator::AssignmentOperator;
pub use comment::Comment;
pub use document::{DocItem, Document};
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
    pub location: Option<Range>,
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

/// Represents a text position in a text file, with line and character
#[derive(Debug, Clone, Default, Copy)]
pub struct Position {
    /// The line that the position is pointing at
    pub line: u32,
    /// The character withing the line that the position is pointing at
    pub char: u32,
}

impl Position {
    /// Creates a position from a line number, and a character number
    #[must_use]
    pub const fn new(line: u32, char: u32) -> Self {
        Self { line, char }
    }
}

/// Location of an error, as a span between `start` and `end`
#[derive(Debug, Clone, Default, Copy)]
pub struct Range {
    /// Position of the start of the error
    pub start: Position,
    /// Position of the end of the error
    pub end: Position,
}

impl Display for Range {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.end.line - self.start.line > 0 {
            write!(
                f,
                "[{}, {}] to [{}, {}]",
                self.start.line, self.start.char, self.end.line, self.end.char
            )
        } else {
            write!(
                f,
                "Ln {}, Col {}-{}",
                self.start.line, self.start.char, self.end.char
            )
        }
    }
}
impl From<Pair<'_, Rule>> for Range {
    fn from(rule: Pair<'_, Rule>) -> Self {
        Range::from(&rule)
    }
}

impl From<&Pair<'_, Rule>> for Range {
    fn from(rule: &Pair<'_, Rule>) -> Self {
        let start = rule.line_col();
        let delta_line = rule.as_str().chars().filter(|&c| c == '\n').count();
        let last_line = rule.as_str().split('\n').last();
        let col = last_line.map_or(0, |ll| ll.chars().count());
        Range {
            start: Position::new(start.0 as u32, start.1 as u32),
            end: Position::new(
                (start.0 + delta_line) as u32,
                if delta_line > 0 {
                    col as u32
                } else {
                    (start.1 + col) as u32
                },
            ),
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
                    .map_or(String::new(), |loc| format!(" at {loc}"))
            ),
            Reason::Unknown => write!(
                f,
                "UNKNOWN ERROR. source: {}{}",
                self.source_text,
                self.location
                    .map_or(String::new(), |loc| format!(" at {loc}"))
            ),
        }
    }
}

impl From<pest::error::Error<Rule>> for Error {
    fn from(value: pest::error::Error<Rule>) -> Self {
        Error {
            reason: Reason::Pest(Box::new(value.clone())),
            source_text: value.to_string(),
            location: Some(
                value
                    .line_col
                    .try_into()
                    .map_or_else(|_| Range::default(), |it| it),
            ),
        }
    }
}

impl TryFrom<pest::error::LineColLocation> for Range {
    type Error = TryFromIntError;
    fn try_from(value: pest::error::LineColLocation) -> Result<Range, TryFromIntError> {
        match value {
            pest::error::LineColLocation::Pos(pos) => Ok(Self {
                start: Position {
                    line: u32::try_from(pos.0)?,
                    char: u32::try_from(pos.1)?,
                },
                end: Position {
                    line: u32::try_from(pos.0)?,
                    char: u32::try_from(pos.1)?,
                },
            }),
            pest::error::LineColLocation::Span(start, end) => Ok(Self {
                start: Position {
                    line: u32::try_from(start.0)?,
                    char: u32::try_from(start.1)?,
                },
                end: Position {
                    line: u32::try_from(end.0)?,
                    char: u32::try_from(end.1)?,
                },
            }),
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
