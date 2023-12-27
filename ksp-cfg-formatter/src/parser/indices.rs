use super::{
    parser_helpers::{expect, range_wrap},
    Ranged, {ASTParse, IResult, LocatedSpan},
};
use nom::{
    branch::alt,
    character::complete::{char, digit1, none_of},
    combinator::{map, map_res, opt, value},
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

impl ASTParse<'_> for Index {
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

impl ASTParse<'_> for ArrayIndex {
    fn parse(input: LocatedSpan) -> IResult<Ranged<ArrayIndex>> {
        let array_index = pair(
            expect(
                alt((
                    value(None, char('*')),
                    map_res(digit1, |n: LocatedSpan| n.fragment().parse().map(Some)),
                )),
                "Expected index, or *",
            ),
            opt(preceded(
                char(','),
                expect(none_of("]"), "Expected char between `,` and closing `]`"),
            )),
        );
        range_wrap(map(
            delimited(
                char('['),
                array_index,
                expect(char(']'), "Expected closing `]`"),
            ),
            |inner| ArrayIndex {
                index: inner.0.unwrap_or_default(),
                separator: inner.1.unwrap_or_default(),
            },
        ))(input)
    }
}
