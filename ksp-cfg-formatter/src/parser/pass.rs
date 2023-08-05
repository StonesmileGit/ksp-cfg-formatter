use std::{convert::Infallible, fmt::Display};

use pest::iterators::Pair;

use super::Rule;

/// Which pass a patch should run on
#[derive(Default, Debug, Clone, Copy, PartialEq)]
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
    /// Final is run last
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

impl<'a> TryFrom<Pair<'a, Rule>> for Pass<'a> {
    type Error = Infallible;

    fn try_from(rule: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        assert!(&rule.clone().into_inner().count().eq(&1));
        let inner = rule.into_inner().next().unwrap();
        match inner.as_rule() {
            Rule::firstPassBlock => Ok(Pass::First),
            Rule::beforePass => Ok(Pass::Before(inner.into_inner().next().unwrap().as_str())),
            Rule::forPass => Ok(Pass::For(inner.into_inner().next().unwrap().as_str())),
            Rule::afterPass => Ok(Pass::After(inner.into_inner().next().unwrap().as_str())),
            Rule::lastPass => Ok(Pass::Last(inner.into_inner().next().unwrap().as_str())),
            Rule::finalPassBlock => Ok(Pass::Final),
            _ => panic!("rule not covered: {:?}", inner.as_rule()),
        }
    }
}
