use pest::iterators::Pair;
use std::fmt::Display;

use super::{Error, Rule};

/// The different kinds of operations that can be done
#[derive(Debug, Clone, Default)]
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

impl TryFrom<Pair<'_, Rule>> for Operator {
    type Error = Error;

    fn try_from(rule: Pair<'_, Rule>) -> Result<Self, Self::Error> {
        match rule.as_str() {
            "" => Ok(Self::None),
            "@" => Ok(Self::Edit),
            "%" => Ok(Self::EditOrCreate),
            "&" => Ok(Self::CreateIfNotFound),
            "+" => Ok(Self::Copy),
            "!" => Ok(Self::Delete),
            "-" => Ok(Self::DeleteAlt),
            "|" => Ok(Self::Rename),
            str => Err(Error {
                source_text: str.to_string(),
                location: Some(rule.into()),
                reason: super::Reason::Custom("Parsing of operator failed".to_string()),
            }),
        }
    }
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
