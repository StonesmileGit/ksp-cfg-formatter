use super::{
    parser_helpers::{debug_fn, expect, non_empty, range_wrap},
    Ranged, {ASTParse, IResult, LocatedSpan},
};
use itertools::Itertools;
use nom::{
    branch::alt,
    bytes::complete::{is_a, tag, tag_no_case},
    character::complete::{anychar, char, line_ending},
    combinator::{map, opt, peek, recognize, value},
    multi::{many1, many_till, separated_list1},
    sequence::{delimited, tuple},
};
use nom_unicode::complete::alphanumeric1;
use std::fmt::Display;

/// Predicate to filter nodes for which to run an operation
#[derive(Debug, Clone)]
pub enum HasPredicate<'a> {
    /// Enum variant for a predicate relating to a node
    NodePredicate {
        /// If true, the node should not be present for the predicate to be satisfied
        negated: bool,
        /// Type of the node, eg: `PART`
        node_type: &'a str,
        /// Optional name of the node e.g: `[part_name]`
        name: Option<&'a str>,
        /// Optional HAS-block to further match on content of node
        has_block: Option<Ranged<HasBlock<'a>>>,
    },
    /// Enum variant for a predicate relating to a variable
    KeyPredicate {
        /// If true, the variable should not be present for the predicate to be satisfied
        negated: bool,
        /// Variable name to check for
        key: &'a str,
        /// Optional value of the variable to check for
        value: Option<Ranged<&'a str>>,
        /// Match type, `<`, ` `, `>`
        match_type: MatchType,
    },
}

impl<'a> Display for HasPredicate<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HasPredicate::NodePredicate {
                negated,
                node_type,
                name,
                has_block,
            } => write!(
                f,
                "{}{}{}{}",
                if *negated { "!" } else { "@" },
                node_type,
                name.map_or_else(String::new, |name| format!("[{name}]")),
                has_block
                    .clone()
                    .map_or_else(String::new, |has_block| has_block.to_string())
            ),
            HasPredicate::KeyPredicate {
                negated,
                key,
                value,
                match_type,
            } => write!(
                f,
                "{}{}{}",
                if *negated { "~" } else { "#" },
                key,
                value
                    .as_ref()
                    .map_or_else(String::new, |value| format!("[{match_type}{value}]"))
            ),
        }
    }
}

/// Enum for the type of comparison to perform on a value
#[derive(Default, Debug, Clone)]
pub enum MatchType {
    /// match the value literally
    #[default]
    Literal,
    /// a value greater than the specified value is a match
    GreaterThan,
    /// a value less than the specified value is a match
    LessThan,
}

impl Display for MatchType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MatchType::Literal => write!(f, ""),
            MatchType::GreaterThan => write!(f, ">"),
            MatchType::LessThan => write!(f, "<"),
        }
    }
}

/// Contains a `Vec` of all the predicates to be combined using logical ANDs. All predicates have to be satisfied for the node to be a match
#[derive(Debug, Clone, Default)]
pub struct HasBlock<'a> {
    /// The predicates that are combined with logical ANDs
    pub predicates: Vec<Ranged<HasPredicate<'a>>>,
}

impl<'a> Display for HasBlock<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.predicates.is_empty() {
            return write!(f, "");
        }
        write!(f, ":HAS[{}]", self.predicates.iter().format(","))
    }
}

impl<'a> ASTParse<'a> for HasBlock<'a> {
    fn parse(input: LocatedSpan<'a>) -> IResult<Ranged<HasBlock<'a>>> {
        range_wrap(map(
            delimited(
                tag_no_case(":HAS["),
                debug_fn(
                    expect(
                        separated_list1(alt((char('&'), char(','))), HasPredicate::parse),
                        "Expected has predicate",
                    ),
                    "Got has predicates",
                    true,
                ),
                expect(char(']'), "Expected closing `]`"),
            ),
            |inner| HasBlock {
                predicates: inner.unwrap_or_default(),
            },
        ))(input)
    }
}

impl<'a> ASTParse<'a> for HasPredicate<'a> {
    fn parse(input: LocatedSpan<'a>) -> IResult<Ranged<HasPredicate<'a>>> {
        let has_value = range_wrap(delimited(
            char('['),
            opt(non_empty(recognize(many_till(
                anychar,
                peek(alt((line_ending::<LocatedSpan, _>, tag("]"), tag("//")))),
            )))),
            expect(char(']'), "Expected closing `]`"),
        ));
        let value_determinative = expect(
            alt((value(false, char('#')), value(true, char('~')))),
            "Expected # or ~",
        );
        let value_constraint = map(
            tuple((
                value_determinative,
                identifier,
                debug_fn(opt(has_value), "Got value", true),
            )),
            |inner: (
                Option<bool>,
                LocatedSpan,
                Option<Ranged<Option<LocatedSpan>>>,
            )| {
                HasPredicate::KeyPredicate {
                    negated: inner.0.unwrap_or_default(),
                    key: inner.1.fragment(),
                    value: inner.2.map(|s| s.map(|s| s.map_or("", |s| s.fragment()))),
                    match_type: MatchType::Literal,
                }
            },
        );

        let name_constraint = delimited(
            char('['),
            recognize(many1(alt((alphanumeric1, is_a("/_-?*.|"))))),
            expect(char(']'), "Expected closing `]`"),
        );
        let node_determinative = expect(
            alt((value(false, char('@')), value(true, char('!')))),
            "Expected @ or !",
        );
        let node_constraint = map(
            tuple((
                node_determinative,
                identifier,
                opt(name_constraint),
                opt(HasBlock::parse),
            )),
            |inner| HasPredicate::NodePredicate {
                negated: inner.0.unwrap_or_default(),
                node_type: inner.1.fragment(),
                name: inner.2.map(|s| *s.fragment()),
                has_block: inner.3,
            },
        );

        range_wrap(alt((node_constraint, value_constraint)))(input)
    }
}

fn identifier(input: LocatedSpan) -> IResult<LocatedSpan> {
    recognize(many1(alt((alphanumeric1, is_a("-_.+*?")))))(input)
}

#[cfg(test)]
mod tests {
    use crate::parser::State;

    use super::*;
    #[test]
    fn test_has() {
        let input = ":HAS[#key[value]]";
        let res = HasBlock::parse(LocatedSpan::new_extra(input, State::default()));

        match res {
            Ok(it) => assert_eq!(input, it.1.to_string()),
            Err(err) => panic!("{}", err),
        }
    }
}
