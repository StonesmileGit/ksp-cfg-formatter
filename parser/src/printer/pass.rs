use std::{convert::Infallible, fmt::Display};

use pest::iterators::Pair;

use crate::reader::Rule;

#[derive(Default, Debug, Clone)]
pub enum Pass<'a> {
    #[default]
    Default,
    First,
    Before(&'a str),
    For(&'a str),
    After(&'a str),
    Last(&'a str),
    Final,
}

impl<'a> Display for Pass<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Pass::Default => write!(f, ""),
            Pass::First => write!(f, ":FIRST"),
            Pass::Before(mod_name) => write!(f, ":BEFORE[{}]", mod_name),
            Pass::For(mod_name) => write!(f, ":FOR[{}]", mod_name),
            Pass::After(mod_name) => write!(f, ":AFTER[{}]", mod_name),
            Pass::Last(mod_name) => write!(f, ":LAST[{}]", mod_name),
            Pass::Final => write!(f, "FINAL"),
        }
    }
}

impl<'a> TryFrom<Pair<'a, Rule>> for Pass<'a> {
    type Error = Infallible;

    fn try_from(rule: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        // dbg!(&rule);
        assert!(&rule.clone().into_inner().into_iter().count().eq(&1));
        let inner = rule.into_inner().next().unwrap();
        match inner.as_rule() {
            Rule::forPass => Ok(Pass::For(inner.into_inner().next().unwrap().as_str())),
            Rule::beforePass => Ok(Pass::Before(inner.into_inner().next().unwrap().as_str())),
            Rule::firstPassBlock => Ok(Pass::First),
            Rule::lastPass => Ok(Pass::Last(inner.into_inner().next().unwrap().as_str())),
            _ => panic!("rule not covered: {:?}", inner.as_rule()),
        }
    }
}
