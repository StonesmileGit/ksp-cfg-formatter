use std::fmt::Display;

use itertools::Itertools;
use nom::{
    branch::alt,
    bytes::complete::{is_a, is_not, tag},
    combinator::{map, opt, recognize, value},
    multi::{many0, many1, separated_list1},
    sequence::{delimited, pair, terminated, tuple},
};
use nom_unicode::complete::alphanumeric1;

use super::{
    nom::{
        utils::{debug_fn, expect, range_wrap},
        CSTParse, IResult, LocatedSpan,
    },
    Ranged,
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
    pub start: Option<Ranged<PathStart>>,
    /// Segments of the path, separated by `/`
    pub segments: Vec<Ranged<PathSegment<'a>>>,
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

impl<'a> CSTParse<'a, Ranged<Path<'a>>> for Path<'a> {
    fn parse(input: LocatedSpan<'a>) -> IResult<Ranged<Path<'a>>> {
        let start = opt(PathStart::parse);
        let segments = many0(PathSegment::parse);
        let path = pair(
            debug_fn(start, "got path start", true),
            debug_fn(segments, "got path segments", true),
        );
        range_wrap(map(path, |inner| Path {
            start: inner.0,
            segments: inner.1,
        }))(input)
    }
}

impl CSTParse<'_, Ranged<PathStart>> for PathStart {
    fn parse(input: LocatedSpan<'_>) -> IResult<Ranged<PathStart>> {
        range_wrap(alt((
            value(PathStart::TopLevel, tag("@")),
            value(PathStart::CurrentTopLevel, tag("/")),
        )))(input)
    }
}

impl<'a> CSTParse<'a, Ranged<PathSegment<'a>>> for PathSegment<'a> {
    fn parse(input: LocatedSpan<'a>) -> IResult<Ranged<PathSegment<'a>>> {
        // path         = ${ ("@" | "/")? ~ (path_segment ~ "/")* }
        // path_segment =  { ".." | identifier ~ ("[" ~ nameBlock ~ "]")? }
        let node = recognize(many1(alt((
            alphanumeric1::<LocatedSpan, _>,
            is_a("-_.+*?"),
        ))));
        let name = opt(delimited(
            tag("["),
            // TODO: is a list allowed here?
            recognize(separated_list1(tag("|"), is_not("|]"))),
            expect(tag("]"), "Expected closing `]`"),
        ));
        let segment = tuple((node, name));
        let dot_dot = map(tag(".."), |_| PathSegment::DotDot);
        let node_name = map(segment, |inner| PathSegment::NodeName {
            node: inner.0.fragment(),
            name: inner.1.map(|s| *s.fragment()),
            // TODO: Add index support
            index: None,
        });
        range_wrap(terminated(alt((dot_dot, node_name)), tag("/")))(input)
    }
}

#[cfg(test)]
mod tests {

    use crate::parser::nom::{LocatedSpan, State};

    use super::*;
    #[test]
    fn test_path() {
        let input = "@PART[RO-M55]/";
        let res = Path::parse(LocatedSpan::new_extra(input, State::default()));

        match res {
            Ok(it) => assert_eq!(input, it.1.to_string()),
            Err(err) => panic!("{}", err),
        }
    }
}
