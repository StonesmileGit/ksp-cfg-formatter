use nom::{
    bytes::complete::tag,
    combinator::{map, value},
};
use pest::iterators::Pair;
use std::fmt::Display;

use super::{nom::CSTParse, Error, Range, Rule};

/// Struct holding info about the operator
#[derive(Debug, Clone, Default)]
pub struct Operator {
    kind: OperatorKind,
    range: Range,
}

/// The different kinds of operations that can be done
#[derive(Debug, Clone, Default, Copy)]
pub enum OperatorKind {
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

impl Operator {
    /// Get the range the operator spans
    #[must_use]
    pub const fn get_pos(&self) -> super::Range {
        self.range
    }
    /// Get what kind the operator is
    #[must_use]
    pub const fn get_kind(&self) -> OperatorKind {
        self.kind
    }
}

impl TryFrom<Pair<'_, Rule>> for Operator {
    type Error = Error;

    fn try_from(rule: Pair<'_, Rule>) -> Result<Self, Self::Error> {
        let range = Range::from(&rule);
        match rule.as_str() {
            "" => Ok(Self {
                kind: OperatorKind::None,
                range,
            }),
            "@" => Ok(Self {
                kind: OperatorKind::Edit,
                range,
            }),
            "%" => Ok(Self {
                kind: OperatorKind::EditOrCreate,
                range,
            }),
            "&" => Ok(Self {
                kind: OperatorKind::CreateIfNotFound,
                range,
            }),
            "+" => Ok(Self {
                kind: OperatorKind::Copy,
                range,
            }),
            "!" => Ok(Self {
                kind: OperatorKind::Delete,
                range,
            }),
            "-" => Ok(Self {
                kind: OperatorKind::DeleteAlt,
                range,
            }),
            "|" => Ok(Self {
                kind: OperatorKind::Rename,
                range,
            }),
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
        match self.kind {
            OperatorKind::None => write!(f, ""),
            OperatorKind::Edit => write!(f, "@"),
            OperatorKind::EditOrCreate => write!(f, "%"),
            OperatorKind::Copy => write!(f, "+"),
            OperatorKind::Delete => write!(f, "!"),
            OperatorKind::DeleteAlt => write!(f, "-"),
            OperatorKind::CreateIfNotFound => write!(f, "&"),
            OperatorKind::Rename => write!(f, "|"),
        }
    }
}

impl CSTParse<'_, Operator> for Operator {
    fn parse(input: super::nom::LocatedSpan) -> super::nom::IResult<Operator> {
        let operator = nom::branch::alt((
            value(OperatorKind::Edit, tag("@")),
            value(OperatorKind::EditOrCreate, tag("%")),
            value(OperatorKind::Copy, tag("+")),
            value(OperatorKind::Delete, tag("!")),
            value(OperatorKind::DeleteAlt, tag("-")),
            value(OperatorKind::CreateIfNotFound, tag("&")),
            value(OperatorKind::Rename, tag("|")),
            // value(OperatorKind::None, tag("")),
        ));
        map(operator, |inner| Operator {
            kind: inner,
            // FIXME: Range is default
            range: Range::default(),
        })(input)
    }
}
