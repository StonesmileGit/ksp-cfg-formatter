use super::{
    ASTPrint, ArrayIndex, AssignmentOperator, Comment, Error, Index, NeedsBlock, Operator, Path,
    Rule,
};
use pest::iterators::Pair;

/// Assignment operation
#[derive(Debug, Default, Clone)]
pub struct KeyVal<'a> {
    /// Optional path to the variable
    pub path: Option<Path<'a>>,
    /// Optional operator
    pub operator: Option<Operator>,
    /// name of the variable
    pub key: &'a str,
    /// Optional NEEDS block
    pub needs: Option<NeedsBlock<'a>>,
    /// Optional index
    pub index: Option<Index>,
    /// Optional array-index
    pub array_index: Option<ArrayIndex>,
    /// The assignment operator between the variable and the value
    pub assignment_operator: AssignmentOperator,
    /// The value to use in the assignment
    pub val: &'a str,
    /// Optional trailing comment
    pub comment: Option<Comment<'a>>,
}

impl<'a> TryFrom<Pair<'a, Rule>> for KeyVal<'a> {
    type Error = Error;

    fn try_from(rule: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        let pairs = rule.into_inner();
        let mut key_val = KeyVal::default();
        for pair in pairs {
            match pair.as_rule() {
                Rule::value => key_val.val = pair.as_str(),
                Rule::Comment => {
                    key_val.comment =
                        Some(Comment::try_from(pair).expect("Parsing a comment is Infallable"));
                }
                Rule::assignmentOperator => {
                    key_val.assignment_operator = AssignmentOperator::try_from(pair)?;
                }
                Rule::needsBlock => key_val.needs = Some(NeedsBlock::try_from(pair)?),
                Rule::index => {
                    key_val.index = Some(super::indices::Index::try_from(pair)?);
                }
                Rule::arrayIndex => {
                    key_val.array_index = Some(super::indices::ArrayIndex::try_from(pair)?);
                }
                Rule::operator => {
                    key_val.operator = Some(Operator::try_from(pair)?);
                }
                Rule::keyIdentifier => key_val.key = pair.as_str().trim(),
                Rule::path => {
                    key_val.path =
                        Some(Path::try_from(pair).expect("Parsing a path is currently Infallable"));
                }
                _ => unreachable!(),
            }
        }
        if key_val.comment.is_none() {
            key_val.val = key_val.val.trim();
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
            self.path
                .clone()
                .map_or_else(String::new, |p| p.to_string()),
            self.operator.clone().unwrap_or_default(),
            self.key,
            self.needs.clone().map_or(String::new(), |n| n.to_string()),
            self.index.map_or_else(String::new, |i| i.to_string()),
            self.array_index.map_or_else(String::new, |i| i.to_string()),
            self.assignment_operator,
            self.val,
            self.comment.map_or_else(String::new, |c| c.to_string()),
            line_ending
        )
    }
}
