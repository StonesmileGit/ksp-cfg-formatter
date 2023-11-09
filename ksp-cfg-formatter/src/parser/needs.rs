use std::fmt::Display;

use itertools::Itertools;
use nom::{
    branch::alt,
    bytes::complete::{is_a, tag_no_case},
    character::complete::{char, one_of},
    combinator::{map, opt, recognize},
    multi::{many1, separated_list1},
    sequence::{delimited, pair},
};
use nom_unicode::complete::alphanumeric1;

use super::{
    nom::{
        utils::{expect, range_wrap},
        CSTParse, IResult, LocatedSpan,
    },
    Ranged,
};

/// Contains a `Vec` of all the clauses to be combined using logical ANDs. All clauses have to be satisfied for the parent operation to be executed
#[derive(Debug, Clone)]
pub struct NeedsBlock<'a> {
    /// The clauses to be combined using logical ANDs
    pub or_clauses: Vec<Ranged<OrClause<'a>>>,
}

impl<'a> Display for NeedsBlock<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, ":NEEDS[{}]", self.or_clauses.iter().format(","))
    }
}

/// Contains a `Vec` of all the clauses to be combined using logical ORs. If any of those clauses are satisfied, the clause is satisfied.
#[derive(Debug, Clone)]
pub struct OrClause<'a> {
    /// The clauses to be combined using logical ORs
    pub mod_clauses: Vec<Ranged<ModClause<'a>>>,
}

impl<'a> Display for OrClause<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.mod_clauses.iter().format("|"))
    }
}

/// A mod that is needed (or not) for the clause to be satisfied
#[derive(Debug, Clone, Default)]
pub struct ModClause<'a> {
    /// If true, the mod should not be present for the clause to be satisfied
    pub negated: bool,
    /// Name of the mod to check for
    pub name: &'a str,
}

impl<'a> Display for ModClause<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", if self.negated { "!" } else { "" }, self.name)
    }
}

impl<'a> CSTParse<'a, Ranged<NeedsBlock<'a>>> for NeedsBlock<'a> {
    fn parse(input: LocatedSpan<'a>) -> IResult<Ranged<NeedsBlock<'a>>> {
        // needsBlock = { ^":NEEDS[" ~ modOrClause ~ (("&" | ",") ~ modOrClause)* ~ "]" }
        range_wrap(map(
            delimited(
                tag_no_case(":NEEDS["),
                expect(
                    separated_list1(one_of("&,"), OrClause::parse),
                    "Expected AND'ed mod",
                ),
                expect(tag_no_case("]"), "Expected closing `]`"),
            ),
            |inner| NeedsBlock {
                or_clauses: inner.unwrap_or_default(),
            },
        ))(input)
    }
}

impl<'a> CSTParse<'a, Ranged<OrClause<'a>>> for OrClause<'a> {
    fn parse(input: LocatedSpan<'a>) -> IResult<Ranged<OrClause<'a>>> {
        // modOrClause = { needsMod ~ ("|" ~ needsMod)* }
        range_wrap(map(
            expect(
                separated_list1(one_of("|"), expect(ModClause::parse, "Expected mod")),
                "Expected OR'd mods",
            ),
            |inner| {
                let mod_clauses = inner
                    .unwrap_or_default()
                    .into_iter()
                    .flatten()
                    .collect_vec();
                OrClause { mod_clauses }
            },
        ))(input)
    }
}

impl<'a> CSTParse<'a, Ranged<ModClause<'a>>> for ModClause<'a> {
    fn parse(input: LocatedSpan<'a>) -> IResult<Ranged<ModClause<'a>>> {
        // needsMod    = { negation? ~ modName }
        // negation    = { "!" }
        let negated = opt(char::<LocatedSpan, _>('!'));
        // modName = { (LETTER | ASCII_DIGIT | "/" | "_" | "-" | "?")+ }
        let mod_name = recognize(many1(alt((alphanumeric1, is_a("/_-?")))));
        let mod_clause = pair(negated, mod_name);
        range_wrap(map(mod_clause, |inner| ModClause {
            negated: inner.0.is_some(),
            name: inner.1.fragment(),
        }))(input)
    }
}

#[cfg(test)]
mod tests {

    use crate::parser::nom::State;

    use super::*;
    #[test]
    fn test_needs() {
        let input = ":NEEDS[mod]";
        let res = NeedsBlock::parse(LocatedSpan::new_extra(input, State::default()));

        match res {
            Ok(it) => {
                let errors = it.0.extra.errors.borrow().clone();
                if errors.len() > 0 {
                    for error in &errors {
                        eprintln!("{:#?}", error);
                    }
                }
                assert_eq!(input, it.1.to_string());
                if errors.len() > 0 {
                    panic!()
                }
            }
            Err(err) => panic!("{}", err),
        }
    }
}
