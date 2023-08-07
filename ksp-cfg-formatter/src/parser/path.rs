use std::{convert::Infallible, fmt::Display};

use itertools::Itertools;
use pest::iterators::Pair;

use super::Rule;

/// Where the path starts from
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathStart {
    /// Path starts from the top level
    //'@'
    TopLevel,
    /// Path starts from the root of the current top level node
    //'/'
    CurrentTopLevel,
}

impl Display for PathStart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PathStart::TopLevel => write!(f, "@"),
            PathStart::CurrentTopLevel => write!(f, "/"),
        }
    }
}

/// Segment of a path, separated by `/`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathSegment<'a> {
    /// Segment is `..`, going up a level
    DotDot,
    /// Name of a node to traverse into
    NodeName {
        /// Node type
        node: &'a str,
        /// Optional node name
        name: Option<&'a str>,
        /// Optional index of the node
        index: Option<i32>,
    },
}

impl<'a> TryFrom<Pair<'a, Rule>> for PathSegment<'a> {
    type Error = Infallible;

    fn try_from(rule: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        if rule.as_str() == ".." {
            Ok(Self::DotDot)
        } else {
            // FIXME: The index should be parsed into the struct
            let mut node = "";
            let mut name = None;

            for pair in rule.into_inner() {
                match pair.as_rule() {
                    Rule::identifier => node = pair.as_str(),
                    Rule::nameBlock => name = Some(pair.as_str()),
                    _ => todo!(),
                }
            }
            Ok(Self::NodeName {
                node,
                name,
                index: None,
            })
        }
    }
}

impl<'a> Display for PathSegment<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PathSegment::DotDot => write!(f, "../"),
            PathSegment::NodeName { node, name, index } => write!(
                f,
                "{}{}{}/",
                node,
                name.map_or_else(String::new, |name| format!("[{}]", name)),
                index.map_or_else(String::new, |index| index.to_string())
            ),
        }
    }
}

/// A path to a node or a variable
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Path<'a> {
    /// Optional start charecter of the path. Starts in current node if not specified
    pub start: Option<PathStart>,
    /// Segments of the path, separated by `/`
    pub segments: Vec<PathSegment<'a>>,
}

impl<'a> Display for Path<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}",
            self.start
                .clone()
                .map_or_else(String::new, |s| s.to_string()),
            self.segments.iter().format("")
        )
    }
}

impl<'a> TryFrom<Pair<'a, Rule>> for Path<'a> {
    type Error = Infallible;

    fn try_from(rule: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        // dbg!(&rule);
        let text = rule.as_str();
        let mut start = None;
        match text.chars().next() {
            Some('@') => start = Some(PathStart::TopLevel),
            Some('/') => start = Some(PathStart::CurrentTopLevel),
            _ => (),
        };
        let mut segments = vec![];
        for pair in rule.into_inner() {
            match pair.as_rule() {
                Rule::path_segment => segments.push(PathSegment::try_from(pair)?),
                _ => unreachable!(),
            }
        }
        Ok(Path { start, segments })
    }
}
