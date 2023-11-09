use std::fmt::Display;

use nom::{
    branch::alt,
    bytes::complete::{is_a, tag_no_case},
    character::complete::char,
    combinator::{map, recognize},
    multi::many1,
    sequence::delimited,
};
use nom_unicode::complete::alphanumeric1;

use super::{
    nom::{
        utils::{expect, range_wrap},
        CSTParse, IResult, LocatedSpan,
    },
    Ranged,
};

/// Which pass a patch should run on
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pass<'a> {
    /// Patches in First are run first. Ordering: (1)
    First,
    /// Default is run after First, before Before. Ordering: (2)
    #[default]
    Default,
    /// Before is run after Default, before For. Ordering: (3)
    Before(&'a str),
    /// For is run after Before, before After. Ordering: (4)
    For(&'a str),
    /// After is run after For, before Last. Ordering: (5)
    After(&'a str),
    /// Last is run after After, before Final. Ordering: (6)
    Last(&'a str),
    /// Final is run last. Ordering: (7)
    Final,
}

impl<'a> Display for Pass<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Pass::Default => write!(f, ""),
            Pass::First => write!(f, ":FIRST"),
            Pass::Before(mod_name) => write!(f, ":BEFORE[{mod_name}]"),
            Pass::For(mod_name) => write!(f, ":FOR[{mod_name}]"),
            Pass::After(mod_name) => write!(f, ":AFTER[{mod_name}]"),
            Pass::Last(mod_name) => write!(f, ":LAST[{mod_name}]"),
            Pass::Final => write!(f, ":FINAL"),
        }
    }
}

impl<'a> CSTParse<'a, Ranged<Pass<'a>>> for Pass<'a> {
    fn parse(input: LocatedSpan<'a>) -> IResult<Ranged<Pass<'a>>> {
        // firstPassBlock = { ^":FIRST" }
        // beforePass     = { ^":BEFORE[" ~ modName ~ "]" }
        // forPass        = { ^":FOR[" ~ modName ~ "]" }
        // afterPass      = { ^":AFTER[" ~ modName ~ "]" }
        // lastPass       = { ^":LAST[" ~ modName ~ "]" }
        // finalPassBlock = { ^":FINAL" }

        // modName = { (LETTER | ASCII_DIGIT | "/" | "_" | "-" | "?")+ }
        // passBlock      = { firstPassBlock | beforePass | forPass | afterPass | lastPass | finalPassBlock }
        range_wrap(alt((
            map(tag_no_case(":FIRST"), |_| Pass::First),
            map(
                delimited(
                    tag_no_case(":BEFORE["),
                    expect(
                        recognize(many1(alt((alphanumeric1::<LocatedSpan, _>, is_a("/_-?"))))),
                        "Expected pass identifier",
                    ),
                    expect(char(']'), "Expected closing `]`"),
                ),
                |inner| Pass::Before(inner.map_or("", |s| s.fragment())),
            ),
            map(
                delimited(
                    tag_no_case(":FOR["),
                    expect(
                        recognize(many1(alt((alphanumeric1::<LocatedSpan, _>, is_a("/_-?"))))),
                        "Expected pass identifier",
                    ),
                    expect(char(']'), "Expected closing `]`"),
                ),
                |inner| Pass::For(inner.map_or("", |s| s.fragment())),
            ),
            map(
                delimited(
                    tag_no_case(":AFTER["),
                    expect(
                        recognize(many1(alt((alphanumeric1::<LocatedSpan, _>, is_a("/_-?"))))),
                        "Expected pass identifier",
                    ),
                    expect(char(']'), "Expected closing `]`"),
                ),
                |inner| Pass::After(inner.map_or("", |s| s.fragment())),
            ),
            map(
                delimited(
                    tag_no_case(":LAST["),
                    expect(
                        recognize(many1(alt((alphanumeric1::<LocatedSpan, _>, is_a("/_-?"))))),
                        "Expected pass identifier",
                    ),
                    expect(char(']'), "Expected closing `]`"),
                ),
                |inner| Pass::Last(inner.map_or("", |s| s.fragment())),
            ),
            map(tag_no_case(":FINAL"), |_| Pass::Final),
        )))(input)
    }
}
