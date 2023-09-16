use std::{convert::Infallible, fmt::Display};

use itertools::Itertools;
use nom::{
    branch::alt,
    bytes::complete::{is_a, is_not, tag},
    character::complete::alphanumeric1,
    combinator::{map, opt, recognize, value},
    multi::{many0, many1, separated_list1},
    sequence::{delimited, pair, terminated, tuple},
};
use pest::iterators::Pair;

use super::{
    nom::{utils::debug_fn, CSTParse, IResult, LocatedSpan},
    Rule,
};

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
                name.map_or_else(String::new, |name| format!("[{name}]")),
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

impl<'a> CSTParse<'a, Path<'a>> for Path<'a> {
    fn parse(input: LocatedSpan<'a>) -> IResult<Path<'a>> {
        let start = opt(PathStart::parse);
        let segments = many0(PathSegment::parse);
        let path = pair(
            debug_fn(start, "got path start", true),
            debug_fn(segments, "got path segments", true),
        );
        map(path, |inner| Path {
            start: inner.0,
            segments: inner.1,
        })(input)
    }
}

impl CSTParse<'_, PathStart> for PathStart {
    fn parse(input: LocatedSpan<'_>) -> IResult<PathStart> {
        alt((
            value(PathStart::TopLevel, tag("@")),
            value(PathStart::CurrentTopLevel, tag("/")),
        ))(input)
    }
}

impl<'a> CSTParse<'a, PathSegment<'a>> for PathSegment<'a> {
    fn parse(input: LocatedSpan<'a>) -> IResult<PathSegment<'a>> {
        // path         = ${ ("@" | "/")? ~ (path_segment ~ "/")* }
        // path_segment =  { ".." | identifier ~ ("[" ~ nameBlock ~ "]")? }
        let node = recognize(many1(alt((
            alphanumeric1::<LocatedSpan, _>,
            is_a("-_.+*?"),
        ))));
        let name = opt(delimited(
            tag("["),
            recognize(separated_list1(tag("|"), is_not("|]"))),
            tag("]"),
        ));
        let segment = tuple((node, name));
        let dot_dot = value(PathSegment::DotDot, tag(".."));
        let node_name = map(segment, |inner| PathSegment::NodeName {
            node: inner.0.fragment(),
            name: inner.1.map(|s| *s.fragment()),
            index: None,
        });
        terminated(alt((dot_dot, node_name)), tag("/"))(input)
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use crate::parser::nom::{LocatedSpan, State};

    use super::*;
    #[test]
    fn test_path() {
        let input = "@PART[RO-M55]/";
        let res = Path::parse(LocatedSpan::new_extra(
            input,
            State(RefCell::new(Vec::new())),
        ));

        match res {
            Ok(it) => assert_eq!(input, it.1.to_string()),
            Err(err) => panic!("{}", err),
        }
    }
}
