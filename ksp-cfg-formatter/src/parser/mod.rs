use std::fmt::Display;

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
pub enum AstParseError {
    /// Parsing a node or the document failed
    NodeParseError(#[from] node::NodeParseError),
    /// Error from Pest
    Pest(Box<pest::error::Error<Rule>>),
    /// The pest parser found no matching rule
    EmptyDocument,
}

impl Display for AstParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AstParseError::NodeParseError(node) => write!(f, "{node}"),
            AstParseError::Pest(pest) => write!(f, "{pest}"),
            AstParseError::EmptyDocument => write!(f, "The parsed text didn't return any tokens"),
        }
    }
}

impl From<pest::error::Error<Rule>> for AstParseError {
    fn from(value: pest::error::Error<Rule>) -> Self {
        AstParseError::Pest(Box::new(value))
    }
}

#[derive(pest_derive::Parser)]
#[grammar = "grammar.pest"]
pub struct Grammar;
