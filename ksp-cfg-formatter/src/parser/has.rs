use super::{Error, Rule};
use itertools::Itertools;
use pest::iterators::Pair;
use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum HasPredicate<'a> {
    NodePredicate {
        negated: bool,
        node_type: &'a str,
        name: Option<&'a str>,
        has_block: Option<HasBlock<'a>>,
    },
    KeyPredicate {
        negated: bool,
        key: &'a str,
        value: Option<&'a str>,
        match_type: MatchType,
    },
}

impl<'a> Display for HasPredicate<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HasPredicate::NodePredicate {
                negated,
                node_type,
                name,
                has_block,
            } => write!(
                f,
                "{}{}{}{}",
                if *negated { "!" } else { "@" },
                node_type,
                name.map_or_else(|| "", |name| name),
                has_block
                    .clone()
                    .map_or_else(String::new, |has_block| has_block.to_string())
            ),
            HasPredicate::KeyPredicate {
                negated,
                key,
                value,
                match_type,
            } => write!(
                f,
                "{}{}{}",
                if *negated { "~" } else { "#" },
                key,
                value.map_or_else(String::new, |value| format!("[{match_type}{value}]"))
            ),
        }
    }
}

impl<'a> TryFrom<Pair<'a, Rule>> for HasPredicate<'a> {
    type Error = Error;

    fn try_from(rule: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        let first_char = rule.as_str().chars().next().unwrap();
        match first_char {
            // Node
            '@' | '!' => {
                let mut node_type = "";
                let mut name = None;
                let mut has_block = None;
                for rule in rule.into_inner() {
                    match rule.as_rule() {
                        Rule::identifier => node_type = rule.as_str(),
                        Rule::hasNodeName => name = Some(rule.as_str()),
                        Rule::hasBlock => has_block = Some(HasBlock::try_from(rule)?),
                        _ => {
                            return Err(Error {
                                reason: super::Reason::Custom(format!("Unexpected Rule '{:?}' encountered when trying to parse HAS block node predicate", rule.as_rule())),
                                source_text: rule.as_str().to_string(),
                                location: Some(rule.into()),
                            });
                        }
                    };
                }
                Ok(HasPredicate::NodePredicate {
                    negated: first_char.ne(&'@'),
                    node_type,
                    name,
                    has_block,
                })
            }
            // Key
            '#' | '~' => {
                let mut key = "";
                let mut value = None;
                let mut match_type = MatchType::default();
                for rule in rule.into_inner() {
                    match rule.as_rule() {
                        Rule::identifier => key = rule.as_str(),
                        Rule::hasValue => {
                            let mut val = rule.as_str();
                            match val.chars().next() {
                                Some('<') => {
                                    match_type = MatchType::LessThan;
                                    val = &val[1..];
                                }
                                Some('>') => {
                                    match_type = MatchType::GreaterThan;
                                    val = &val[1..];
                                }
                                _ => (),
                            };
                            value = Some(val);
                        }
                        _ => {
                            return Err(Error {
                                reason: super::Reason::Custom(format!("Unexpected Rule '{:?}' encountered when trying to parse HAS block key predicate", rule.as_rule())),
                                source_text: rule.as_str().to_string(),
                                location: Some(rule.into()),
                            });
                        }
                    }
                }
                Ok(HasPredicate::KeyPredicate {
                    negated: first_char.ne(&'#'),
                    key,
                    value,
                    match_type,
                })
            }
            _ => Err(Error {
                reason: super::Reason::Custom(
                    "Unexpected first char encountered when trying to parse HAS block predicate"
                        .to_string(),
                ),
                source_text: rule.as_str().to_string(),
                location: Some(rule.into()),
            }),
        }
    }
}

#[derive(Default, Debug, Clone)]
pub enum MatchType {
    #[default]
    Literal,
    GreaterThan,
    LessThan,
}

impl Display for MatchType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MatchType::Literal => write!(f, ""),
            MatchType::GreaterThan => write!(f, ">"),
            MatchType::LessThan => write!(f, "<"),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct HasBlock<'a> {
    predicates: Vec<HasPredicate<'a>>,
}

impl<'a> Display for HasBlock<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.predicates.is_empty() {
            return write!(f, "");
        }
        write!(f, ":HAS[{}]", self.predicates.iter().format(","))
    }
}

impl<'a> TryFrom<Pair<'a, Rule>> for HasBlock<'a> {
    type Error = Error;

    fn try_from(rule: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        assert!(matches!(rule.as_rule(), Rule::hasBlock));
        let mut has_block = HasBlock::default();
        for rule in rule.into_inner() {
            has_block.predicates.push(HasPredicate::try_from(rule)?);
        }
        Ok(has_block)
    }
}
