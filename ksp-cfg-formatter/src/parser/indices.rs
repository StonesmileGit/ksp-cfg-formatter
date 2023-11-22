use super::{
    parser_helpers::{expect, range_wrap},
    Ranged, {CSTParse, IResult, LocatedSpan},
};
use nom::{
    branch::alt,
    character::complete::{anychar, char, digit1},
    combinator::{map, opt, value},
    sequence::{delimited, pair, preceded},
};
use std::fmt::Display;

/// Selects from multiple matching objects
#[derive(Debug, Clone, Copy)]
pub enum Index {
    /// Operate on all matches, `,*`
    All,
    /// Integer match to operate on. Can be negative to start from back, `,i`
    Number(i32),
}

impl Display for Index {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Index::All => write!(f, ",*"),
            Index::Number(n) => write!(f, ",{n}"),
        }
    }
}

impl CSTParse<'_, Ranged<Index>> for Index {
    fn parse(input: LocatedSpan) -> IResult<Ranged<Index>> {
        // index = { "," ~ ("*" | ("-"? ~ ASCII_DIGIT+)) }
        range_wrap(preceded(
            char(','),
            alt((
                value(Index::All, char('*')),
                map(preceded(opt(char('-')), digit1), |inner: LocatedSpan| {
                    Index::Number(
                        inner
                            .fragment()
                            .parse()
                            .expect("Only digits are allowed to get through the parser"),
                    )
                }),
            )),
        ))(input)
    }
}

/// index in value of variable to operate on
#[derive(Debug, Clone, Copy)]
pub struct ArrayIndex {
    /// Index to operate on, all if `None` (from `*`)
    pub index: Option<i32>,
    /// Char separating the values in the array. `,` if not specified
    pub separator: Option<char>,
}

impl Display for ArrayIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}{}{}]",
            self.index.map_or("*".to_owned(), |index| index.to_string()),
            if self.separator.is_some() { "," } else { "" },
            self.separator
                .map_or_else(String::new, |separator| separator.to_string())
        )
    }
}

impl CSTParse<'_, Ranged<ArrayIndex>> for ArrayIndex {
    fn parse(input: LocatedSpan) -> IResult<Ranged<ArrayIndex>> {
        // arrayIndex = { "[" ~ ("*" | ASCII_DIGIT+) ~ ("," ~ ANY)? ~ "]" }
        let array_index = pair(
            alt((
                value(None, char('*')),
                map(digit1, |n: LocatedSpan| Some(n.fragment().parse().unwrap())),
            )),
            opt(preceded(char(','), anychar)),
        );
        range_wrap(map(
            delimited(
                char('['),
                // TODO: Add "expect" on the index too
                array_index,
                expect(char(']'), "Expected closing `]`"),
            ),
            |inner| ArrayIndex {
                index: inner.0,
                separator: inner.1,
            },
        ))(input)
    }
}
