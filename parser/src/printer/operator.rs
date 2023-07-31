use crate::reader::Rule;
use pest::iterators::Pair;
use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum Operator {
    None,
    Edit,
    EditOrCreate,
    CreateIfNotFound,
    Copy,
    Delete,
    //TODO: Wanted?
    DeleteAlt,
    //TODO: This is technically not allowed in top level nodes
    Rename,
}

#[derive(Debug)]
pub struct OperatorParseError<'a> {
    pub text: &'a str,
}
impl<'a> TryFrom<Pair<'a, Rule>> for Operator {
    type Error = OperatorParseError<'a>;

    fn try_from(rule: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        match rule.as_str() {
            "" => Ok(Self::None),
            "@" => Ok(Self::Edit),
            "%" => Ok(Self::EditOrCreate),
            "&" => Ok(Self::CreateIfNotFound),
            "+" => Ok(Self::Copy),
            "!" => Ok(Self::Delete),
            "-" => Ok(Self::DeleteAlt),
            "|" => Ok(Self::Rename),
            _ => Err(OperatorParseError {
                text: rule.as_str(),
            }),
        }
    }
}

impl Default for Operator {
    fn default() -> Self {
        Self::None
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
