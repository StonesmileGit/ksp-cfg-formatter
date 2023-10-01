use super::{
    nom::{utils::range_wrap, CSTParse, IResult, LocatedSpan},
    Ranged,
};
use nom::{branch::alt, bytes::complete::tag, combinator::value};
use std::fmt::Display;

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
impl CSTParse<'_, Ranged<AssignmentOperator>> for AssignmentOperator {
    fn parse(input: LocatedSpan) -> IResult<Ranged<AssignmentOperator>> {
        range_wrap(alt((
            value(AssignmentOperator::Add, tag("+=")),
            value(AssignmentOperator::Subtract, tag("-=")),
            value(AssignmentOperator::Multiply, tag("*=")),
            value(AssignmentOperator::Divide, tag("/=")),
            value(AssignmentOperator::Power, tag("!=")),
            value(AssignmentOperator::RegexReplace, tag("^=")),
            value(AssignmentOperator::Assign, tag("=")),
        )))(input)
    }
}
