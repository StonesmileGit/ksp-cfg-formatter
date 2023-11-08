use self::nom::{LocatedSpan, Severity};
use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
};

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
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
pub struct Error {
    /// The severity of the error
    pub severity: Severity,
    /// The reason for the error
    pub reason: Reason,
    /// Optional line/col span indicating the origin of the error
    pub location: Option<Range>,
    /// Source string from which the error occured
    pub source_text: String,
}

/// Reason for the error that occured
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Reason {
    /// Parsing of an int failed
    ParseInt,
    /// Custom error with reason provided
    Custom(String),
    /// Unknown error
    #[default]
    Unknown,
}

/// Wrapper to hold the range that the inner type spans
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Ranged<T> {
    inner: T,
    range: Range,
}

impl<T> Display for Ranged<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

impl<T> Ranged<T> {
    /// Creates a wrapper over the inner item with the range provided
    #[must_use]
    pub const fn new(inner: T, range: Range) -> Self {
        Self { inner, range }
    }

    /// Get the range the operator spans
    #[must_use]
    pub const fn get_range(&self) -> Range {
        self.range
    }

    /// Map a Ranged<T> to a Ranged<U> using the passed function
    #[must_use]
    pub fn map<U, F>(self, f: F) -> Ranged<U>
    where
        F: FnOnce(T) -> U,
    {
        Ranged {
            inner: f(self.inner),
            range: self.range,
        }
    }
}

impl<T> AsRef<T> for Ranged<T> {
    fn as_ref(&self) -> &T {
        &self.inner
    }
}

impl<T> Deref for Ranged<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for Ranged<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

/// Represents a text position in a text file, with line and character
#[derive(Debug, Clone, Default, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Position {
    /// The line that the position is pointing at
    pub line: u32,
    /// The character withing the line that the position is pointing at
    pub col: u32,
}

impl Position {
    /// Creates a position from a line number, and a character number
    #[must_use]
    pub const fn new(line: u32, col: u32) -> Self {
        Self { line, col }
    }

    /// Creates a Position from a `LocatedSpan`
    pub fn from_located_span(span: &LocatedSpan) -> Self {
        Self::new(
            span.location_line(),
            span.get_utf8_column()
                .try_into()
                .expect("both usize and u32 should be large enough"),
        )
    }
}

/// Location of an error, as a span between `start` and `end`
#[derive(Debug, Clone, Default, Copy, PartialEq, Eq)]
pub struct Range {
    /// Position of the start of the error
    pub start: Position,
    /// Position of the end of the error
    pub end: Position,
}

impl Range {
    /// Creates a range from starting and ending line and column
    #[must_use]
    pub const fn new(start_line: u32, start_col: u32, end_line: u32, end_col: u32) -> Self {
        Self {
            start: Position {
                line: start_line,
                col: start_col,
            },
            end: Position {
                line: end_line,
                col: end_col,
            },
        }
    }

    /// Creates a range from starting and ending `LocatedSpan`
    pub fn from_locations(start: &LocatedSpan, end: &LocatedSpan) -> Self {
        Self {
            start: Position::from_located_span(start),
            end: Position::from_located_span(end),
        }
    }

    /// Creates a Range with the end set to the same as the start of the current range
    #[must_use]
    pub const fn to_start(&self) -> Self {
        Self {
            start: self.start,
            end: self.start,
        }
    }
}

impl std::ops::Add for Range {
    type Output = Range;

    fn add(self, rhs: Self) -> Self::Output {
        let start = self.start.min(rhs.start);
        let end = self.end.max(rhs.end);
        Self { start, end }
    }
}

impl Display for Range {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.end.line - self.start.line > 0 {
            write!(
                f,
                "[{}, {}] to [{}, {}]",
                self.start.line, self.start.col, self.end.line, self.end.col
            )
        } else {
            write!(
                f,
                "Ln {}, Col {}-{}",
                self.start.line, self.start.col, self.end.col
            )
        }
    }
}

impl<'a> From<LocatedSpan<'a>> for Range {
    fn from(value: LocatedSpan) -> Self {
        let start = Position::from_located_span(&value);
        let delta_lines: u32 = value
            .fragment()
            .chars()
            .filter(|&c| c == '\n')
            .count()
            .try_into()
            .expect("usize and u32 should both be large enough");
        let last_line = value.fragment().split('\n').last();
        let col: u32 = last_line.map_or(0, |ll| ll.chars().count().try_into().unwrap());
        let end = Position {
            line: start.line + delta_lines,
            col: if delta_lines > 0 {
                col
            } else {
                start.col + col
            },
        };
        Self { start, end }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.reason {
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

impl From<nom::Error> for Error {
    fn from(value: nom::Error) -> Self {
        Self {
            reason: Reason::Custom(value.message),
            location: Some(value.range),
            source_text: value.source,
            severity: value.severity,
        }
    }
}
