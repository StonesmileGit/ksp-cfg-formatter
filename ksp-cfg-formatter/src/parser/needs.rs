use std::fmt::Display;

use itertools::Itertools;
use pest::iterators::Pair;

use super::{Error, Rule};

/// Contains a `Vec` of all the clauses to be combined using logical ANDs. All clauses have to be satisfied for the parent operation to be executed
#[derive(Debug, Clone)]
pub struct NeedsBlock<'a> {
    /// The clauses to be combined using logical ANDs
    pub or_clauses: Vec<OrClause<'a>>,
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

/// Contains a `Vec` of all the clauses to be combined using logical ORs. If any of those clauses are satisfied, the clause is satisfied.
#[derive(Debug, Clone)]
pub struct OrClause<'a> {
    /// The clauses to be combined using logical ORs
    pub mod_clauses: Vec<ModClause<'a>>,
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
                    reason: super::Reason::Custom(
                        "Unexpected rule enountered when parsing 'or clause'".to_string(),
                    ),
                });
            }
        }
        Ok(OrClause { mod_clauses })
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
                        source_text: pair.as_str().to_string(),
                        reason: super::Reason::Custom(
                            "Unexpected rule enountered when parsing 'mod clause'".to_string(),
                        ),
                        location: Some(pair.into()),
                    });
                }
            }
        }
        Ok(mod_clause)
    }
}
