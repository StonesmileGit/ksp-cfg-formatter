use std::{fmt::Display, str::FromStr};

#[derive(Debug, Clone)]
pub enum Operator {
    None,
    Edit,
    EditOrCreate,
    Copy,
    Delete,
}

#[derive(Debug)]
pub struct OperatorParseError;
impl FromStr for Operator {
    type Err = OperatorParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "" => Ok(Self::None),
            "@" => Ok(Self::Edit),
            "%" => Ok(Self::EditOrCreate),
            "+" => Ok(Self::Copy),
            "!" => Ok(Self::Delete),
            _ => Err(OperatorParseError),
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
        }
    }
}
