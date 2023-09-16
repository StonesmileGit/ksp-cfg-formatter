use super::{
    nom::{CSTParse, IResult, LocatedSpan},
    Error, Rule,
};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{anychar, digit1},
    combinator::{map, opt, value},
    sequence::{delimited, pair, preceded},
};
use pest::iterators::Pair;
use std::fmt::Display;

/// Selects from multiple matching objects
#[derive(Debug, Clone, Copy)]
pub enum Index {
    /// Operate on all matches, `,*`
    All,
    /// Integer match to operate on. Can be negative to start from back, `,i`
    Number(i32),
}

impl<'a> TryFrom<Pair<'a, Rule>> for Index {
    type Error = Error;

    fn try_from(rule: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        let s = rule.as_str();
        let a = &s[1..];
        match a {
            "*" => Ok(Self::All),
            _ => Ok(Self::Number(match a.parse() {
                Ok(i) => i,
                Err(_) => {
                    return Err(super::Error {
                        location: Some(rule.into()),
                        reason: super::Reason::ParseInt,
                        source_text: s.to_string(),
                    })
                }
            })),
        }
    }
}

impl Display for Index {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Index::All => write!(f, ",*"),
            Index::Number(n) => write!(f, ",{n}"),
        }
    }
}

impl CSTParse<'_, Index> for Index {
    fn parse(input: LocatedSpan) -> IResult<Index> {
        // index = { "," ~ ("*" | ("-"? ~ ASCII_DIGIT+)) }
        preceded(
            tag(","),
            alt((
                value(Index::All, tag("*")),
                // TODO: Allow negative numbers
                map(digit1, |inner: LocatedSpan| {
                    Index::Number(
                        inner
                            .fragment()
                            .parse()
                            .expect("Only digits are allowed to get through the parser"),
                    )
                }),
            )),
        )(input)
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

impl<'a> TryFrom<Pair<'a, Rule>> for ArrayIndex {
    type Error = Error;

    fn try_from(rule: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        let s = rule.as_str();
        let trimmed = &s[1..s.len() - 1];
        let split = trimmed.split_once(',');
        let first = split.map_or(trimmed, |split| split.0);
        let second = split.map(|split| split.1);
        let index = match first {
            "*" => None,
            _ => Some(match first.parse() {
                Ok(i) => i,
                Err(_) => {
                    return Err(super::Error {
                        location: Some(rule.into()),
                        reason: super::Reason::ParseInt,
                        source_text: s.to_string(),
                    })
                }
            }),
        };
        if let Some(scnd) = second {
            if scnd.chars().count() > 1 {
                return Err(Error {
                    reason: super::Reason::Custom(
                        "Error while trying to parse array index delimiter, too many chars found"
                            .to_string(),
                    ),
                    source_text: rule.as_str().to_string(),
                    location: Some(rule.into()),
                });
            }
        }
        let separator = second.map(|s| s.chars().next().unwrap());
        Ok(ArrayIndex { index, separator })
    }
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

impl CSTParse<'_, ArrayIndex> for ArrayIndex {
    fn parse(input: LocatedSpan) -> IResult<ArrayIndex> {
        // arrayIndex = { "[" ~ ("*" | ASCII_DIGIT+) ~ ("," ~ ANY)? ~ "]" }
        let array_index = pair(
            alt((
                value(None, tag("*")),
                map(digit1, |n: LocatedSpan| Some(n.fragment().parse().unwrap())),
            )),
            opt(preceded(tag(","), anychar)),
        );
        map(delimited(tag("["), array_index, tag("]")), |inner| {
            ArrayIndex {
                index: inner.0,
                separator: inner.1,
            }
        })(input)
    }
}
