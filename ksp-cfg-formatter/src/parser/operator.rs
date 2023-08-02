use crate::Rule;
use pest::iterators::Pair;
use std::fmt::Display;

#[derive(Debug, Clone, Default)]
pub enum Operator {
    #[default]
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

#[derive(Debug, Clone, thiserror::Error)]
pub struct OperatorParseError {
    pub text: String,
}

impl Display for OperatorParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}
impl TryFrom<Pair<'_, Rule>> for Operator {
    type Error = OperatorParseError;

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
            _ => Err(OperatorParseError {
                text: rule.as_str().to_string(),
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
