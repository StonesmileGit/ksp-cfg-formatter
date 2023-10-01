use nom::{bytes::complete::tag, combinator::value};
use std::fmt::Display;

use super::{
    nom::{utils::range_wrap, CSTParse},
    Ranged,
};

/// The different kinds of operations that can be done
#[derive(Debug, Clone, Default, Copy)]
pub enum Operator {
    /// No operator
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
    /// Wanted?
    //TODO: Wanted?
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

impl CSTParse<'_, Ranged<Operator>> for Operator {
    fn parse(input: super::nom::LocatedSpan) -> super::nom::IResult<Ranged<Operator>> {
        // TODO: Maybe it woud be better to return a Range based on the consumed span?
        let operator = nom::branch::alt((
            value(Operator::Edit, tag("@")),
            value(Operator::EditOrCreate, tag("%")),
            value(Operator::Copy, tag("+")),
            value(Operator::Delete, tag("!")),
            value(Operator::DeleteAlt, tag("-")),
            value(Operator::CreateIfNotFound, tag("&")),
            value(Operator::Rename, tag("|")),
            // value(OperatorKind::None, tag("")),
        ));
        range_wrap(operator)(input)
    }
}
