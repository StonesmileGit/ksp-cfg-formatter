use std::fmt::Display;

use itertools::Itertools;
use nom::{
    branch::alt,
    bytes::complete::{is_a, tag, tag_no_case},
    character::complete::{alphanumeric1, one_of},
    combinator::{map, opt, recognize},
    multi::{many1, separated_list1},
    sequence::{delimited, pair},
};
use pest::iterators::Pair;

use super::{
    nom::{utils::expect, CSTParse, IResult, LocatedSpan},
    Error, Range, Rule,
};

/// Contains a `Vec` of all the clauses to be combined using logical ANDs. All clauses have to be satisfied for the parent operation to be executed
#[derive(Debug, Clone)]
pub struct NeedsBlock<'a> {
    /// The clauses to be combined using logical ANDs
    pub or_clauses: Vec<OrClause<'a>>,
    _range: Range,
}

impl<'a> Display for NeedsBlock<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, ":NEEDS[{}]", self.or_clauses.iter().format(","))
    }
}

impl<'a> TryFrom<Pair<'a, Rule>> for NeedsBlock<'a> {
    type Error = Error;

    fn try_from(rule: Pair<'a, Rule>) -> Result<Self, Error> {
        let range = Range::from(&rule);
        let mut or_clauses = vec![];
        for pair in rule.into_inner() {
            if pair.as_rule() == Rule::modOrClause {
                or_clauses.push(OrClause::try_from(pair)?);
            } else {
                let rule_name = pair.as_rule();
                panic!("Got unexpected rule: {rule_name:?}");
            }
        }
        Ok(NeedsBlock {
            or_clauses,
            _range: range,
        })
    }
}

/// Contains a `Vec` of all the clauses to be combined using logical ORs. If any of those clauses are satisfied, the clause is satisfied.
#[derive(Debug, Clone)]
pub struct OrClause<'a> {
    /// The clauses to be combined using logical ORs
    pub mod_clauses: Vec<ModClause<'a>>,
    _range: Range,
}

impl<'a> Display for OrClause<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.mod_clauses.iter().format("|"))
    }
}

impl<'a> TryFrom<Pair<'a, Rule>> for OrClause<'a> {
    type Error = Error;

    fn try_from(rule: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        let range = Range::from(&rule);
        let mut mod_clauses = vec![];
        for pair in rule.into_inner() {
            if pair.as_rule() == Rule::needsMod {
                mod_clauses.push(ModClause::try_from(pair)?);
            } else {
                return Err(Error {
                    source_text: pair.as_str().to_string(),
                    location: Some(pair.into()),
                    reason: super::Reason::Custom(
                        "Unexpected rule enountered when parsing 'or clause'".to_string(),
                    ),
                });
            }
        }
        Ok(OrClause {
            mod_clauses,
            _range: range,
        })
    }
}

/// A mod that is needed (or not) for the clause to be satisfied
#[derive(Debug, Clone, Default)]
pub struct ModClause<'a> {
    /// If true, the mod should not be present for the clause to be satisfied
    pub negated: bool,
    /// Name of the mod to check for
    pub name: &'a str,
    _range: Range,
}

impl<'a> Display for ModClause<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", if self.negated { "!" } else { "" }, self.name)
    }
}

impl<'a> TryFrom<Pair<'a, Rule>> for ModClause<'a> {
    type Error = Error;

    fn try_from(rule: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        let range = Range::from(&rule);
        let mut mod_clause = ModClause {
            _range: range,
            ..Default::default()
        };
        for pair in rule.into_inner() {
            match pair.as_rule() {
                Rule::negation => mod_clause.negated = true,
                Rule::modName => mod_clause.name = pair.as_str(),
                rl => {
                    return Err(Error {
                        source_text: pair.as_str().to_string(),
                        reason: super::Reason::Custom(format!(
                            "Unexpected rule enountered when parsing 'mod clause', found '{rl:?}'"
                        )),
                        location: Some(pair.into()),
                    });
                }
            }
        }
        Ok(mod_clause)
    }
}

impl<'a> CSTParse<'a, NeedsBlock<'a>> for NeedsBlock<'a> {
    fn parse(input: LocatedSpan<'a>) -> IResult<NeedsBlock<'a>> {
        // needsBlock = { ^":NEEDS[" ~ modOrClause ~ (("&" | ",") ~ modOrClause)* ~ "]" }
        map(
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
                _range: Range::default(),
            },
        )(input)
    }
}

impl<'a> CSTParse<'a, OrClause<'a>> for OrClause<'a> {
    fn parse(input: LocatedSpan<'a>) -> IResult<OrClause<'a>> {
        // modOrClause = { needsMod ~ ("|" ~ needsMod)* }
        map(
            expect(
                separated_list1(one_of("|"), ModClause::parse),
                "Expected OR'd mods",
            ),
            |inner| OrClause {
                mod_clauses: inner.unwrap_or_default(),
                _range: Range::default(),
            },
        )(input)
    }
}

impl<'a> CSTParse<'a, ModClause<'a>> for ModClause<'a> {
    fn parse(input: LocatedSpan<'a>) -> IResult<ModClause<'a>> {
        // needsMod    = { negation? ~ modName }
        // negation    = { "!" }
        let negated = opt(tag::<_, LocatedSpan, _>("!"));
        // modName = { (LETTER | ASCII_DIGIT | "/" | "_" | "-" | "?")+ }
        let mod_name = recognize(many1(alt((alphanumeric1, is_a("/_-?")))));
        let mod_clause = pair(negated, mod_name);
        map(mod_clause, |inner| ModClause {
            negated: inner.0.is_some(),
            name: inner.1.fragment(),
            _range: Range::default(),
        })(input)
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use crate::parser::nom::State;

    use super::*;
    #[test]
    fn test_needs() {
        let input = ":NEEDS[]";
        let res = NeedsBlock::parse(LocatedSpan::new_extra(
            input,
            State(RefCell::new(Vec::new())),
        ));

        match res {
            Ok(it) => {
                let errors = it.0.extra.0.into_inner();
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
