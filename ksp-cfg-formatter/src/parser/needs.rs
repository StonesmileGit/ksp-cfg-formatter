use std::fmt::Display;

use itertools::Itertools;
use pest::iterators::Pair;

use crate::Rule;

use super::Error;

#[derive(Debug, Clone)]
pub struct NeedsBlock<'a> {
    or_clauses: Vec<OrClause<'a>>,
}

impl<'a> Display for NeedsBlock<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, ":NEEDS[{}]", self.or_clauses.iter().format(","))
    }
}

impl<'a> TryFrom<Pair<'a, Rule>> for NeedsBlock<'a> {
    type Error = Error;

    fn try_from(rule: Pair<'a, Rule>) -> Result<Self, Error> {
        let mut or_clauses = vec![];
        for pair in rule.into_inner() {
            if pair.as_rule() == Rule::modOrClause {
                or_clauses.push(OrClause::try_from(pair)?);
            } else {
                let rule_name = pair.as_rule();
                panic!("Got unexpected rule: {rule_name:?}");
            }
        }
        Ok(NeedsBlock { or_clauses })
    }
}
#[derive(Debug, Clone)]
struct OrClause<'a> {
    mod_clauses: Vec<ModClause<'a>>,
}

impl<'a> Display for OrClause<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.mod_clauses.iter().format("|"))
    }
}

impl<'a> TryFrom<Pair<'a, Rule>> for OrClause<'a> {
    type Error = Error;

    fn try_from(rule: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        let mut mod_clauses = vec![];
        for pair in rule.into_inner() {
            if pair.as_rule() == Rule::needsMod {
                mod_clauses.push(ModClause::try_from(pair)?);
            } else {
                return Err(Error {
                    source_text: pair.as_str().to_string(),
                    location: Some(pair.into()),
                    reason: super::Reason::Unknown,
                });
            }
        }
        Ok(OrClause { mod_clauses })
    }
}

#[derive(Debug, Clone, Default)]
struct ModClause<'a> {
    negated: bool,
    name: &'a str,
}

impl<'a> Display for ModClause<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", if self.negated { "!" } else { "" }, self.name)
    }
}

impl<'a> TryFrom<Pair<'a, Rule>> for ModClause<'a> {
    type Error = Error;

    fn try_from(rule: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        let mut mod_clause = ModClause::default();
        for pair in rule.into_inner() {
            match pair.as_rule() {
                Rule::negation => mod_clause.negated = true,
                Rule::modName => mod_clause.name = pair.as_str(),
                _ => {
                    return Err(Error {
                        location: None,
                        reason: super::Reason::Unknown,
                        source_text: pair.as_str().to_string(),
                    });
                }
            }
        }
        Ok(mod_clause)
    }
}
