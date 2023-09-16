use super::{
    nom::{
        utils::{debug_fn, expect},
        CSTParse, IResult, LocatedSpan,
    },
    Error, Rule,
};
use itertools::Itertools;
use nom::{
    branch::alt,
    bytes::complete::{is_a, tag, tag_no_case},
    character::complete::{alphanumeric1, anychar, line_ending, one_of},
    combinator::{map, not, opt, peek, recognize, value},
    multi::{many0, many1, many_till, separated_list1},
    sequence::{delimited, tuple},
};
use pest::iterators::Pair;
use std::fmt::Display;

/// Predicate to filter nodes for which to run an operation
#[derive(Debug, Clone)]
pub enum HasPredicate<'a> {
    /// Enum variant for a predicate relating to a node
    NodePredicate {
        /// If true, the node should not be present for the predicate to be satisfied
        negated: bool,
        /// Type of the node, eg: `PART`
        node_type: &'a str,
        /// Optional name of the node e.g: `[part_name]`
        name: Option<&'a str>,
        /// Optional HAS-block to further match on content of node
        has_block: Option<HasBlock<'a>>,
    },
    /// Enum variant for a predicate relating to a variable
    KeyPredicate {
        /// If true, the variable should not be present for the predicate to be satisfied
        negated: bool,
        /// Variable name to check for
        key: &'a str,
        /// Optional value of the variable to check for
        value: Option<&'a str>,
        /// Match type, `<`, ` `, `>`
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
                        rl => {
                            return Err(Error {
                                reason: super::Reason::Custom(format!("Unexpected Rule '{rl:?}' encountered when trying to parse HAS block node predicate")),
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
                        rl => {
                            return Err(Error {
                                reason: super::Reason::Custom(format!("Unexpected Rule '{rl:?}' encountered when trying to parse HAS block key predicate")),
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
            ch => Err(Error {
                reason: super::Reason::Custom(
                    format!("Unexpected first char encountered when trying to parse HAS block predicate, found '{ch}'"),
                ),
                source_text: rule.as_str().to_string(),
                location: Some(rule.into()),
            }),
        }
    }
}

/// Enum for the type of comparison to perform on a value
#[derive(Default, Debug, Clone)]
pub enum MatchType {
    /// match the value literally
    #[default]
    Literal,
    /// a value greater than the specified value is a match
    GreaterThan,
    /// a value less than the specified value is a match
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

/// Contains a `Vec` of all the predicates to be combined using logical ANDs. All predicates have to be satisfied for the node to be a match
#[derive(Debug, Clone, Default)]
pub struct HasBlock<'a> {
    /// The predicates that are combined with logical ANDs
    pub predicates: Vec<HasPredicate<'a>>,
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

impl<'a> CSTParse<'a, HasBlock<'a>> for HasBlock<'a> {
    fn parse(input: LocatedSpan<'a>) -> IResult<HasBlock<'a>> {
        // hasBlock     = { ^":HAS[" ~ (hasBlockPart ~ (("&" | ",") ~ hasBlockPart)*) ~ "]" }
        map(
            delimited(
                tag_no_case(":HAS["),
                debug_fn(
                    expect(
                        separated_list1(alt((tag("&"), tag(","))), HasPredicate::parse),
                        "Expected has predicate",
                    ),
                    "Got has predicates",
                    true,
                ),
                tag("]"),
            ),
            |inner| HasBlock {
                predicates: inner.unwrap_or_default(),
            },
        )(input)
    }
}

impl<'a> CSTParse<'a, HasPredicate<'a>> for HasPredicate<'a> {
    fn parse(input: LocatedSpan<'a>) -> IResult<HasPredicate<'a>> {
        // // TODO: Is this correct?
        // hasValue = { (!(Newline | "]" | "//") ~ ANY)* }
        let has_value = delimited(
            tag("["),
            recognize(many_till(
                anychar,
                peek(alt((line_ending::<LocatedSpan, _>, tag("]"), tag("//")))),
            )),
            tag("]"),
        );
        // identifier = ${ ("-" | "_" | "." | "+" | "*" | "?" | LETTER | ASCII_DIGIT)+ }
        let identifier = recognize(many1(alt((alphanumeric1, is_a("-_.+*?")))));
        // hasKey = _{ ("#" | "~") ~ identifier ~ ("[" ~ hasValue ~ "]")? }
        let has_key = tuple((
            alt((value(false, tag("#")), value(true, tag("~")))),
            identifier,
            // opt(has_value),
            debug_fn(expect(has_value, "Expected value"), "Got value", true),
        ));
        let key_map = map(has_key, |inner| HasPredicate::KeyPredicate {
            negated: inner.0,
            key: inner.1.fragment(),
            value: inner.2.map(|s| *s.fragment()),
            match_type: MatchType::Literal,
        });
        // hasNodeName =  { ("[" ~ (LETTER | ASCII_DIGIT | "/" | "_" | "-" | "?" | "*" | "." | "|")+ ~ "]") }
        // TODO: remove ´delimited´ when old parser is removed
        let has_node_name = recognize(delimited(
            tag("["),
            recognize(many1(alt((alphanumeric1, is_a("/_-?*.|"))))),
            tag("]"),
        ));
        // FIXME
        let identifier = recognize(many1(alt((alphanumeric1, is_a("-_.+*?")))));
        // hasNode     = _{ ("@" | "!" | "-") ~ identifier ~ hasNodeName? ~ hasBlock? }
        let has_node = tuple((
            alt((value(false, tag("@")), value(true, one_of("!-")))),
            identifier,
            opt(has_node_name),
            opt(HasBlock::parse),
        ));
        let node_map = map(has_node, |inner| HasPredicate::NodePredicate {
            negated: inner.0,
            node_type: inner.1.fragment(),
            name: inner.2.map(|s| *s.fragment()),
            has_block: inner.3,
        });
        // hasBlockPart = { hasNode | hasKey }
        alt((node_map, key_map))(input)
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use crate::parser::nom::State;

    use super::*;
    #[test]
    fn test_has() {
        let input = ":HAS[#key[value]]";
        let res = HasBlock::parse(LocatedSpan::new_extra(
            input,
            State(RefCell::new(Vec::new())),
        ));

        match res {
            Ok(it) => assert_eq!(input, it.1.to_string()),
            Err(err) => panic!("{}", err),
        }
    }
}
