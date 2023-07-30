use std::{fmt::Display, str::FromStr};

#[derive(Debug, Default)]
pub enum AssignmentOperator {
    #[default]
    Assign,
    Multiply,
    Divide,
    Add,
    Subtract,
    Power,
    RegexReplace,
}

pub struct ParseAssignmentError;
impl FromStr for AssignmentOperator {
    type Err = ParseAssignmentError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "=" => Ok(AssignmentOperator::Assign),
            "*=" => Ok(AssignmentOperator::Multiply),
            "/=" => Ok(AssignmentOperator::Divide),
            "+=" => Ok(AssignmentOperator::Add),
            "-=" => Ok(AssignmentOperator::Subtract),
            "!=" => Ok(AssignmentOperator::Power),
            "^=" => Ok(AssignmentOperator::RegexReplace),
            _ => Err(ParseAssignmentError),
        }
    }
}

impl Display for AssignmentOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssignmentOperator::Assign => write!(f, "="),
            AssignmentOperator::Multiply => write!(f, "*="),
            AssignmentOperator::Divide => write!(f, "/="),
            AssignmentOperator::Add => write!(f, "+="),
            AssignmentOperator::Subtract => write!(f, "-="),
            AssignmentOperator::Power => write!(f, "!="),
            AssignmentOperator::RegexReplace => write!(f, "^="),
        }
    }
}
