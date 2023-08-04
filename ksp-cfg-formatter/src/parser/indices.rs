use crate::Rule;
use pest::iterators::Pair;
use std::fmt::Display;

use super::Error;

#[derive(Debug, Clone, Copy)]
pub enum Index {
    All,
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

#[derive(Debug, Clone, Copy)]
pub struct ArrayIndex {
    index: Option<i32>,
    separator: Option<char>,
}

impl<'a> TryFrom<Pair<'a, Rule>> for ArrayIndex {
    type Error = Error;

    fn try_from(rule: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        let s = rule.as_str();
        let trimmed = &s[1..s.len() - 1];
        let mut a = trimmed.split(',');
        let b = a.next().unwrap();
        let index = match b {
            "*" => None,
            _ => Some(match b.parse() {
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
        let separator = a.next().map(|s| s.chars().next().unwrap());
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
