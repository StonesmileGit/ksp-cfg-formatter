use super::{
    assignment_operator::AssignmentOperator, comment::Comment, operator::Operator, ASTPrint,
};

#[derive(Debug)]
pub struct KeyVal {
    pub operator: Option<Operator>,
    pub key: String,
    pub needs: Option<String>,
    // TODO: Replace with an enum
    pub index: Option<String>,
    // TODO: Replace with a struct
    pub array_index: Option<String>,
    pub assignment_operator: AssignmentOperator,
    pub val: String,
    pub comment: Option<Comment>,
}

impl ASTPrint for KeyVal {
    fn ast_print(&self, depth: usize, indentation: &str, line_ending: &str, _: bool) -> String {
        let indentation = indentation.repeat(depth);
        format!(
            "{}{}{}{}{}{} {} {}{}{}",
            indentation,
            self.operator.clone().unwrap_or_default(),
            self.key,
            self.needs.clone().unwrap_or_default(),
            self.index.clone().unwrap_or_default(),
            self.array_index.clone().unwrap_or_default(),
            self.assignment_operator,
            self.val,
            self.comment.clone().unwrap_or_default(),
            line_ending
        )
    }
}
