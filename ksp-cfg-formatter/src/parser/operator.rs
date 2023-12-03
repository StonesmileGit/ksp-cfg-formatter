use nom::{branch::alt, character::complete::char, combinator::value};
use std::fmt::Display;

use super::{parser_helpers::range_wrap, ASTParse, Ranged};

/// The different kinds of operations that can be done
#[derive(Debug, Clone, Default, Copy)]
pub enum Operator {
    /// No operator. This is never parsed, only used as a default fallback for printing
    #[default]
    None,
    /// Edit an existing node/variable
    Edit,
    /// Edit-or-create a node/variable
    EditOrCreate,
    /// Create a node/value if not found
    CreateIfNotFound,
    /// Copy an existing node/variable
    Copy,
    /// Delete a node/variable
    Delete,
    /// Delete a node/variable (Alt 2)
    DeleteAlt,
    /// Rename a node. Not allowed on top level nodes
    Rename,
}

impl Display for Operator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operator::None => write!(f, ""),
            Operator::Edit => write!(f, "@"),
            Operator::EditOrCreate => write!(f, "%"),
            Operator::Copy => write!(f, "+"),
            Operator::Delete => write!(f, "!"),
            Operator::DeleteAlt => write!(f, "-"),
            Operator::CreateIfNotFound => write!(f, "&"),
            Operator::Rename => write!(f, "|"),
        }
    }
}

impl ASTParse<'_> for Operator {
    fn parse(input: super::LocatedSpan) -> super::IResult<Ranged<Operator>> {
        let operator = alt((
            value(Operator::Edit, char('@')),
            value(Operator::EditOrCreate, char('%')),
            value(Operator::Copy, char('+')),
            value(Operator::Delete, char('!')),
            value(Operator::DeleteAlt, char('-')),
            value(Operator::CreateIfNotFound, char('&')),
            value(Operator::Rename, char('|')),
        ));
        range_wrap(operator)(input)
    }
}
