use super::{
    assignment_operator::AssignmentOperator,
    comment::Comment,
    indices::{ArrayIndex, Index},
    operator::Operator,
    ASTPrint,
};

#[derive(Debug, Default)]
pub struct KeyVal {
    // TODO: Replace with struct
    pub path: Option<String>,
    pub operator: Option<Operator>,
    pub key: String,
    pub needs: Option<String>,
    pub index: Option<Index>,
    pub array_index: Option<ArrayIndex>,
    pub assignment_operator: AssignmentOperator,
    pub val: String,
    pub comment: Option<Comment>,
}

impl ASTPrint for KeyVal {
    fn ast_print(&self, depth: usize, indentation: &str, line_ending: &str, _: bool) -> String {
        let indentation = indentation.repeat(depth);
        format!(
            "{}{}{}{}{}{}{}{} {} {}{}{}",
            indentation,
            if self.path.is_some() { "*" } else { "" },
            self.path.clone().unwrap_or_default(),
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
