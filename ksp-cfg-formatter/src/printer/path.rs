use std::{convert::Infallible, fmt::Display};

use itertools::Itertools;
use pest::iterators::Pair;

use crate::reader::Rule;

#[derive(Debug, Clone)]
pub enum PathStart {
    //'@'
    TopLevel,
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

#[derive(Debug, Clone)]
pub enum PathSegment<'a> {
    DotDot,
    NodeName {
        node: &'a str,
        name: Option<&'a str>,
        index: Option<i32>,
    },
}

impl<'a> TryFrom<Pair<'a, Rule>> for PathSegment<'a> {
    type Error = Infallible;

    fn try_from(rule: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        dbg!(&rule);
        let res = if rule.as_str() == ".." {
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
        };
        dbg!(&res);
        res
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
                name.map_or_else(String::new, ToString::to_string),
                index.map_or_else(String::new, |index| index.to_string())
            ),
        }
    }
}

// TODO: Is this the best way to do it, since only the last segment can be/has to be a key?
// Turns out the grammar is made to not include the key in the path...
#[derive(Debug, Clone)]
pub struct Path<'a> {
    start: Option<PathStart>,
    segments: Vec<PathSegment<'a>>,
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
