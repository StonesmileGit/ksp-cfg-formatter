use super::{nom::CSTParse, Rule};
use nom::{bytes::complete::tag, combinator::value};
use pest::iterators::Pair;
use std::fmt::Display;

use super::Error;

/// Assignment operator in a key-val
#[derive(Debug, Default, Clone, Copy)]
pub enum AssignmentOperator {
    /// Default assignment, `=`
    #[default]
    Assign,
    /// Multiply the variable by the value, `*=`
    Multiply,
    /// Divide the variable by the value, `/=`
    Divide,
    /// Increment the variable by the value, `+=`
    Add,
    /// Decrement the variable by the value, `-=`
    Subtract,
    /// Raise the variable by the value, `!=`
    Power,
    /// Regex operation, `^=`
    RegexReplace,
}

impl<'a> TryFrom<Pair<'a, Rule>> for AssignmentOperator {
    type Error = Error;

    fn try_from(rule: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        match rule.as_str() {
            "=" => Ok(AssignmentOperator::Assign),
            "*=" => Ok(AssignmentOperator::Multiply),
            "/=" => Ok(AssignmentOperator::Divide),
            "+=" => Ok(AssignmentOperator::Add),
            "-=" => Ok(AssignmentOperator::Subtract),
            "!=" => Ok(AssignmentOperator::Power),
            "^=" => Ok(AssignmentOperator::RegexReplace),
            str => Err(Error {
                source_text: rule.as_str().to_string(),
                location: Some(rule.into()),
                reason: super::Reason::Custom(format!("Unexpected character combination encountered when parsing 'Assignment Operator', found '{str}'")),
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

impl CSTParse<'_, AssignmentOperator> for AssignmentOperator {
    fn parse(input: super::nom::LocatedSpan) -> super::nom::IResult<AssignmentOperator> {
        nom::branch::alt((
            value(AssignmentOperator::Add, tag("+=")),
            value(AssignmentOperator::Subtract, tag("-=")),
            value(AssignmentOperator::Multiply, tag("*=")),
            value(AssignmentOperator::Divide, tag("/=")),
            value(AssignmentOperator::Power, tag("!=")),
            value(AssignmentOperator::RegexReplace, tag("^=")),
            value(AssignmentOperator::Assign, tag("=")),
        ))(input)
    }
}
