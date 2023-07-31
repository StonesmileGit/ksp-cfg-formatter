use crate::reader::Rule;
use itertools::Itertools;
use pest::iterators::Pair;
use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum Predicate<'a> {
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

impl<'a> Display for Predicate<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Predicate::NodePredicate {
                negated,
                node_type,
                name,
                has_block,
            } => write!(
                f,
                "{}{}{}{}",
                if *negated { "!" } else { "@" },
                node_type,
                if name.is_some() {
                    format!("{}", name.unwrap())
                } else {
                    "".to_owned()
                },
                if has_block.is_some() {
                    has_block.clone().unwrap().to_string()
                } else {
                    "".to_owned()
                }
            ),
            Predicate::KeyPredicate {
                negated,
                key,
                value,
                match_type,
            } => write!(
                f,
                "{}{}{}",
                if *negated { "~" } else { "#" },
                key,
                if value.is_some() {
                    format!("[{}{}]", match_type.to_string(), value.unwrap())
                } else {
                    "".to_owned()
                }
            ),
        }
    }
}

impl<'a> TryFrom<Pair<'a, Rule>> for Predicate<'a> {
    type Error = HasBlockError;

    fn try_from(rule: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        // dbg!(&rule);
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
                        // _ => panic!("node rule: {}", rule),
                        _ => return Err(HasBlockError),
                    };
                }
                Ok(Predicate::NodePredicate {
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
                            match val.chars().nth(0) {
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
                        // _ => panic!("key rule: {}", rule),
                        _ => return Err(HasBlockError),
                    }
                }
                Ok(Predicate::KeyPredicate {
                    negated: first_char.ne(&'#'),
                    key,
                    value,
                    match_type,
                })
            }
            // _ => panic!("got char '{}'", first_char),
            _ => Err(HasBlockError),
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
    predicates: Vec<Predicate<'a>>,
}

impl<'a> Display for HasBlock<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.predicates.len() == 0 {
            return write!(f, "");
        }
        write!(f, ":HAS[{}]", self.predicates.iter().format(","))
    }
}

pub struct HasBlockError;
impl<'a> TryFrom<Pair<'a, Rule>> for HasBlock<'a> {
    type Error = HasBlockError;

    fn try_from(rule: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        assert!(matches!(rule.as_rule(), Rule::hasBlock));
        let mut has_block = HasBlock::default();
        for rule in rule.into_inner() {
            has_block.predicates.push(Predicate::try_from(rule)?);
        }
        Ok(has_block)
    }
}
