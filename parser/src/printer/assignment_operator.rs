use crate::reader::Rule;
use pest::iterators::Pair;
use std::fmt::Display;

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

pub struct ParseAssignmentError<'a> {
    pub text: &'a str,
}
impl<'a> TryFrom<Pair<'a, Rule>> for AssignmentOperator {
    type Error = ParseAssignmentError<'a>;

    fn try_from(rule: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        match rule.as_str() {
            "=" => Ok(AssignmentOperator::Assign),
            "*=" => Ok(AssignmentOperator::Multiply),
            "/=" => Ok(AssignmentOperator::Divide),
            "+=" => Ok(AssignmentOperator::Add),
            "-=" => Ok(AssignmentOperator::Subtract),
            "!=" => Ok(AssignmentOperator::Power),
            "^=" => Ok(AssignmentOperator::RegexReplace),
            _ => Err(ParseAssignmentError {
                text: rule.as_str(),
            }),
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
