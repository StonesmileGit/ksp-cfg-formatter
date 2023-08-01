use std::fmt::Display;

use itertools::Itertools;
use pest::iterators::Pair;

use crate::reader::Rule;

#[derive(Debug, Clone)]
pub struct NeedsBlock<'a> {
    or_clauses: Vec<OrClause<'a>>,
}

impl<'a> Display for NeedsBlock<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, ":NEEDS[{}]", self.or_clauses.iter().format(","))
    }
}

pub struct NeedsBlockError {
    pub text: String,
}
impl<'a> TryFrom<Pair<'a, Rule>> for NeedsBlock<'a> {
    type Error = NeedsBlockError;

    fn try_from(rule: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        dbg!(&rule);
        let mut or_clauses = vec![];
        for pair in rule.into_inner() {
            match pair.as_rule() {
                Rule::modOrClause => or_clauses.push(OrClause::try_from(pair)?),
                _ => {
                    let rule_name = pair.as_rule();
                    panic!("Got unexpected rule: {:?}", rule_name);
                }
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
    type Error = NeedsBlockError;

    fn try_from(rule: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        let mut mod_clauses = vec![];
        for pair in rule.into_inner() {
            match pair.as_rule() {
                Rule::needsMod => mod_clauses.push(ModClause::try_from(pair)?),
                _ => {
                    let rule_name = pair.as_rule();
                    panic!("Got unexpected rule: {:?}", rule_name);
                }
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
    type Error = NeedsBlockError;

    fn try_from(rule: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        dbg!(&rule);
        let mut mod_clause = ModClause::default();
        for pair in rule.into_inner() {
            match pair.as_rule() {
                Rule::negation => mod_clause.negated = true,
                Rule::modName => mod_clause.name = pair.as_str(),
                _ => {
                    return Err(NeedsBlockError {
                        text: "Got unexpected rule".to_owned(),
                    })
                }
            }
        }
        Ok(mod_clause)
    }
}
