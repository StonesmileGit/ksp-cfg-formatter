use std::ops::Deref;

use super::{
    nom::{
        utils::{debug_fn, ignore_line_ending, range_wrap, ws},
        CSTParse, IResult, LocatedSpan,
    },
    ASTPrint, ArrayIndex, AssignmentOperator, Comment, Error, Index, NeedsBlock, Operator, Path,
    Range, Ranged, Rule,
};
use nom::{
    branch::alt,
    bytes::complete::{is_a, tag},
    character::complete::{alphanumeric1, anychar, none_of, space1},
    combinator::{eof, map, opt, peek, recognize},
    multi::{many1, many_till, separated_list1},
    sequence::{preceded, terminated, tuple},
};
use pest::iterators::Pair;

/// Assignment operation
#[derive(Debug, Default, Clone)]
pub struct KeyVal<'a> {
    /// Optional path to the variable
    pub path: Option<Ranged<Path<'a>>>,
    /// Optional operator
    pub operator: Option<Ranged<Operator>>,
    /// name of the variable
    pub key: Ranged<&'a str>,
    /// Optional NEEDS block
    pub needs: Option<Ranged<NeedsBlock<'a>>>,
    /// Optional index
    pub index: Option<Ranged<Index>>,
    /// Optional array-index
    pub array_index: Option<Ranged<ArrayIndex>>,
    key_padding: Option<String>,
    /// The assignment operator between the variable and the value
    pub assignment_operator: Ranged<AssignmentOperator>,
    /// The value to use in the assignment
    // FIXME: The value has the trailing whitespace in case of a comment. Split into a separate field
    pub val: Ranged<&'a str>,
    /// Optional trailing comment
    pub comment: Option<Ranged<Comment<'a>>>,
}

impl<'a> KeyVal<'a> {
    pub(crate) fn left_side(&self) -> String {
        format!(
            "{}{}{}{}{}{}{}",
            if self.path.is_some() { "*" } else { "" },
            self.path
                .clone()
                .map_or_else(String::new, |p| p.to_string()),
            self.operator.clone().unwrap_or_default(),
            self.key,
            self.needs.clone().map_or(String::new(), |n| n.to_string()),
            self.index
                .as_deref()
                .map_or_else(String::new, |i| i.to_string()),
            self.array_index
                .as_deref()
                .map_or_else(String::new, |i| i.to_string()),
        )
    }
    pub(crate) fn set_key_padding(&mut self, n: usize) {
        self.key_padding = Some(" ".repeat(n - self.left_side().len()));
    }
}

impl<'a> TryFrom<(Pair<'a, Rule>, bool)> for Ranged<KeyVal<'a>> {
    type Error = Error;

    fn try_from(rule_bool: (Pair<'a, Rule>, bool)) -> Result<Self, Self::Error> {
        let rule = rule_bool.0;
        let range = Range::from(&rule);
        let top_level = rule_bool.1;
        let pairs = rule.into_inner();
        let mut key_val = KeyVal {
            ..Default::default()
        };
        for pair in pairs {
            match pair.as_rule() {
                Rule::value => key_val.val = Ranged::new(pair.as_str(), pair.into()),
                Rule::Comment => {
                    key_val.comment = Some(
                        Ranged::<Comment>::try_from(pair).expect("Parsing a comment is Infallable"),
                    );
                }
                Rule::assignmentOperator => {
                    key_val.assignment_operator = Ranged::<AssignmentOperator>::try_from(pair)?;
                }
                Rule::needsBlock => key_val.needs = Some(Ranged::<NeedsBlock>::try_from(pair)?),
                Rule::index => {
                    key_val.index = Some(Ranged::<Index>::try_from(pair)?);
                }
                Rule::arrayIndex => {
                    key_val.array_index = Some(Ranged::<ArrayIndex>::try_from(pair)?);
                }
                Rule::operator => {
                    let op = Some(Ranged::<Operator>::try_from(pair.clone())?);
                    if let Some(op) = &op {
                        if top_level && matches!(op.deref(), Operator::Rename) {
                            return Err(Error {
                                reason: crate::parser::Reason::Custom(
                                    "Found rename operator on top level node".to_string(),
                                ),
                                source_text: pair.as_str().to_string(),
                                location: Some(pair.into()),
                            });
                        }
                    }
                    key_val.operator = op;
                }
                Rule::keyIdentifier => key_val.key = Ranged::new(pair.as_str().trim(), pair.into()),
                Rule::path => {
                    key_val.path = Some(
                        Ranged::<Path>::try_from(pair)
                            .expect("Parsing a path is currently Infallable"),
                    );
                }
                _ => unreachable!(),
            }
        }
        if key_val.comment.is_none() {
            *key_val.val = key_val.val.trim();
        }
        Ok(Ranged::new(key_val, range))
    }
}

impl<'a> ASTPrint for KeyVal<'a> {
    fn ast_print(&self, depth: usize, indentation: &str, line_ending: &str, _: bool) -> String {
        let indentation = indentation.repeat(depth);
        format!(
            "{}{}{}{}{}{}{}{}{} {} {}{}{}",
            indentation,
            if self.path.is_some() { "*" } else { "" },
            self.path
                .clone()
                .map_or_else(String::new, |p| p.to_string()),
            self.operator.clone().unwrap_or_default(),
            self.key,
            self.needs.clone().map_or(String::new(), |n| n.to_string()),
            self.index
                .as_deref()
                .map_or_else(String::new, |i| i.to_string()),
            self.array_index
                .as_deref()
                .map_or_else(String::new, |i| i.to_string()),
            self.key_padding.clone().map_or_else(String::new, |p| p),
            self.assignment_operator,
            self.val,
            self.comment
                .as_ref()
                .map_or_else(String::new, |c| c.to_string()),
            line_ending
        )
    }
}

impl<'a> CSTParse<'a, Ranged<KeyVal<'a>>> for KeyVal<'a> {
    fn parse(input: LocatedSpan<'a>) -> IResult<Ranged<KeyVal<'a>>> {
        let path = opt(preceded(tag("*"), Path::parse));
        let operator = opt(Operator::parse);
        // keyIdentifier     = ${ keyIdentifierPart ~ (Whitespace* ~ keyIdentifierPart)* }
        // keyIdentifierPart = _{ ("#" | "_" | "." | (("-" | "+" | "*") ~ !"=") | ("/" ~ !("/" | "=")) | "?" | LETTER | ASCII_DIGIT)+ }
        let key = range_wrap(map(
            recognize(separated_list1(
                space1::<LocatedSpan, _>,
                recognize(many1(alt((
                    is_a("#_.?"),
                    alphanumeric1,
                    terminated(is_a("-+*"), none_of("=")),
                    terminated(tag("/"), none_of("/=")),
                )))),
            )),
            |s| *s.fragment(),
        ));
        let needs = opt(NeedsBlock::parse);
        let index = opt(Index::parse);
        let array_index = opt(ArrayIndex::parse);
        let assignment_operator = ws(AssignmentOperator::parse);
        let value = range_wrap(map(
            ignore_line_ending(recognize(many_till(
                anychar,
                peek(alt((tag("//"), is_a("}\r\n"), eof))),
            ))),
            |s| *s.fragment(),
        ));
        let comment = opt(ignore_line_ending(ws(Comment::parse)));
        let key_val = tuple((
            debug_fn(path, "Got path", true),
            debug_fn(operator, "Got operator", true),
            debug_fn(key, "Got key", true),
            needs,
            index,
            array_index,
            debug_fn(assignment_operator, "Got AOP", true),
            debug_fn(value, "Got value", true),
            comment,
        ));
        range_wrap(map(key_val, |inner| {
            let path = inner.0;
            let operator = inner.1;
            let key = inner.2;
            let needs = inner.3;
            let index = inner.4;
            let array_index = inner.5;
            let key_padding = None;
            let assignment_operator = inner.6;
            let val = inner.7;
            let comment = inner.8;
            // This technically only has to contain the first and last range that is Some, since it's "added up" later
            // dbg!(&inner);
            let mut key_val = KeyVal {
                path,
                operator,
                key,
                needs,
                index,
                array_index,
                key_padding,
                assignment_operator,
                val,
                comment,
            };
            if key_val.comment.is_none() {
                *key_val.val = key_val.val.trim();
            }
            key_val
        }))(input)
    }
}

#[cfg(test)]
mod tests {

    use crate::parser::nom::{LocatedSpan, State};

    use super::*;
    #[test]
    fn test_key_val() {
        let input = "key = val\r\n";
        let res = KeyVal::parse(LocatedSpan::new_extra(input, State::default()));

        match res {
            Ok(it) => assert_eq!(input, it.1.ast_print(0, "\t", "\r\n", false)),
            Err(err) => panic!("{}", err),
        }
    }
    #[test]
    fn test_key_val_2() {
        let input = "*@PART[RO-M55]/deleteMe = true\r\n";
        let res = KeyVal::parse(LocatedSpan::new_extra(input, State::default()));

        match res {
            Ok(it) => assert_eq!(input, it.1.ast_print(0, "\t", "\r\n", false)),
            Err(err) => panic!("{}", err),
        }
    }
}
