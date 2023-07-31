use std::convert::Infallible;

use pest::iterators::Pair;

use crate::reader::Rule;

use super::{
    assignment_operator::AssignmentOperator,
    comment::Comment,
    indices::{ArrayIndex, Index},
    operator::Operator,
    path::Path,
    ASTPrint,
};

#[derive(Debug, Default)]
pub struct KeyVal<'a> {
    pub path: Option<Path<'a>>,
    pub operator: Option<Operator>,
    pub key: String,
    pub needs: Option<String>,
    pub index: Option<Index>,
    pub array_index: Option<ArrayIndex>,
    pub assignment_operator: AssignmentOperator,
    pub val: String,
    pub comment: Option<Comment>,
}

impl<'a> TryFrom<Pair<'a, Rule>> for KeyVal<'a> {
    type Error = Infallible;

    fn try_from(rule: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        let pairs = rule.into_inner();
        let mut key_val = KeyVal::default();
        for pair in pairs {
            match pair.as_rule() {
                Rule::value => key_val.val = pair.as_str().to_string(),
                Rule::Comment => {
                    key_val.comment = Some(Comment::try_from(pair)?);
                }
                Rule::assignmentOperator => {
                    key_val.assignment_operator =
                        super::assignment_operator::AssignmentOperator::try_from(pair)
                            .ok()
                            .unwrap();
                }
                Rule::needsBlock => key_val.needs = Some(pair.as_str().to_string()),
                Rule::index => {
                    let text = pair.as_str().to_string();
                    key_val.index = Some(
                        super::indices::Index::try_from(pair)
                            .unwrap_or_else(|_| panic!("{}", text)),
                    )
                }
                Rule::arrayIndex => {
                    let text = pair.as_str().to_string();
                    key_val.array_index = Some(
                        super::indices::ArrayIndex::try_from(pair)
                            .unwrap_or_else(|_| panic!("{}", text)),
                    )
                }
                Rule::operator => {
                    let text = pair.as_str().to_string();
                    key_val.operator =
                        Some(Operator::try_from(pair).unwrap_or_else(|_| panic!("{}", text)));
                }
                Rule::keyIdentifier => key_val.key = pair.as_str().trim().to_string(),
                Rule::path => key_val.path = Some(Path::try_from(pair)?),
                _ => unreachable!(),
            }
        }
        if key_val.comment.is_none() {
            key_val.val = key_val.val.trim().to_string();
        }
        Ok(key_val)
    }
}

impl<'a> ASTPrint for KeyVal<'a> {
    fn ast_print(&self, depth: usize, indentation: &str, line_ending: &str, _: bool) -> String {
        let indentation = indentation.repeat(depth);
        format!(
            "{}{}{}{}{}{}{}{} {} {}{}{}",
            indentation,
            if self.path.is_some() { "*" } else { "" },
            self.path.clone().map_or("".to_owned(), |p| p.to_string()),
            self.operator.clone().unwrap_or_default(),
            self.key,
            self.needs.clone().unwrap_or_default(),
            self.index.clone().map_or("".to_owned(), |i| i.to_string()),
            self.array_index
                .clone()
                .map_or("".to_owned(), |i| i.to_string()),
            self.assignment_operator,
            self.val,
            self.comment.clone().unwrap_or_default(),
            line_ending
        )
    }
}
