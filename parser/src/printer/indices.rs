use crate::reader::Rule;
use pest::iterators::Pair;
use std::{fmt::Display, num::ParseIntError};

#[derive(Debug, Clone)]
pub enum Index {
    All,
    Number(i32),
}

impl<'a> TryFrom<Pair<'a, Rule>> for Index {
    type Error = ParseIntError;

    fn try_from(rule: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        let s = rule.as_str();
        let a = &s[1..];
        match a {
            "*" => Ok(Self::All),
            _ => Ok(Self::Number(a.parse()?)),
        }
    }
}

impl Display for Index {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Index::All => write!(f, ",*"),
            Index::Number(n) => write!(f, ",{}", n),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ArrayIndex {
    index: Option<i32>,
    separator: Option<char>,
}

impl<'a> TryFrom<Pair<'a, Rule>> for ArrayIndex {
    type Error = ParseIntError;

    fn try_from(rule: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        let s = rule.as_str();
        let trimmed = &s[1..s.len() - 1];
        let mut a = trimmed.split(',');
        let b = a.next().unwrap();
        let index = match b {
            "*" => None,
            _ => Some(b.parse()?),
        };
        let separator = a.next().map(|s| s.chars().next().unwrap());
        Ok(ArrayIndex { index, separator })
    }
}

impl Display for ArrayIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}{}{}]",
            if self.index.is_some() {
                self.index.unwrap().to_string()
            } else {
                "*".to_owned()
            },
            if self.separator.is_some() { "," } else { "" },
            if self.separator.is_some() {
                self.separator.unwrap().to_string()
            } else {
                "".to_owned()
            }
        )
    }
}
