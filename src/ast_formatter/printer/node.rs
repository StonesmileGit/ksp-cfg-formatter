use super::{comment::Comment, indices::Index, ASTPrint, NodeItem};

#[derive(Debug)]
pub struct Node {
    pub operator: Option<String>,
    pub identifier: String,
    pub name: Option<String>,
    pub has: Option<String>,
    pub needs: Option<String>,
    pub pass: Option<String>,
    pub index: Option<Index>,
    pub id_comment: Option<Comment>,
    pub comments_after_newline: Vec<Comment>,
    pub block: Vec<NodeItem>,
    pub trailing_comment: Option<Comment>,
}

impl ASTPrint for Node {
    fn ast_print(
        &self,
        depth: usize,
        indentation: &str,
        line_ending: &str,
        should_collapse: bool,
    ) -> String {
        let mut output = String::new();
        for comment in &self.comments_after_newline {
            output.push_str(
                comment
                    .ast_print(depth, indentation, line_ending, should_collapse)
                    .as_str(),
            );
        }
        let indentation_str = indentation.repeat(depth);
        let complete_node_name = format!(
            "{}{}{}{}{}{}{}",
            self.operator.clone().unwrap_or_default(),
            self.identifier,
            self.name.clone().unwrap_or_default(),
            self.has.clone().unwrap_or_default(),
            self.needs.clone().unwrap_or_default(),
            self.pass.clone().unwrap_or_default(),
            self.index.clone().map_or("".to_owned(), |i| i.to_string()),
        );
        output.push_str(
            match self.block.len() {
                0 if self.id_comment.is_none() => {
                    format!(
                        "{}{} {{}}{}{}",
                        indentation_str,
                        complete_node_name,
                        self.trailing_comment
                            .as_ref()
                            .unwrap_or(&Comment {
                                text: String::new()
                            })
                            .text,
                        line_ending
                    )
                }
                1 if should_collapse && short_node(self) => {
                    format!(
                        "{}{} {{ {} }}{}{}",
                        indentation_str,
                        complete_node_name,
                        self.block
                            .first()
                            .unwrap()
                            .ast_print(0, indentation, "", should_collapse),
                        self.trailing_comment
                            .as_ref()
                            .unwrap_or(&Comment {
                                text: String::new()
                            })
                            .text,
                        line_ending
                    )
                }
                _ => {
                    let mut output = format!(
                        "{}{}{}{}{}{{{}",
                        indentation_str,
                        complete_node_name,
                        self.id_comment
                            .as_ref()
                            .unwrap_or(&Comment {
                                text: String::new()
                            })
                            .text,
                        line_ending,
                        indentation_str,
                        line_ending
                    );
                    for statement in &self.block {
                        output.push_str(
                            statement
                                .ast_print(depth + 1, indentation, line_ending, should_collapse)
                                .as_str(),
                        );
                    }
                    output.push_str(&indentation_str);
                    output.push('}');
                    output.push_str(
                        self.trailing_comment
                            .as_ref()
                            .unwrap_or(&Comment {
                                text: String::new(),
                            })
                            .text
                            .as_str(),
                    );
                    output.push_str(line_ending);
                    output
                }
            }
            .as_str(),
        );
        output
    }
}

fn short_node(arg: &Node) -> bool {
    const MAX_LENGTH: usize = 72;
    if arg.id_comment.is_some() {
        return false;
    }
    let mut len = 7; // Include the opening/closing bracket and spaces around operator
    len += arg.identifier.chars().count();
    if let Some(name) = arg.name.clone() {
        len += name.chars().count();
    }
    match arg.block.first().unwrap() {
        NodeItem::KeyVal(kv) => {
            if kv.operator.is_some() {
                len += 1;
            }
            len += kv.key.chars().count();
            len += kv.assignment_operator.to_string().chars().count();
            len += kv.val.chars().count();
            if kv.comment.is_some() {
                return false;
            };
        }
        _ => return false,
    }
    len <= MAX_LENGTH
}
