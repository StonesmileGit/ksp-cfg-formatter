use self::{comment::Comment, key_val::KeyVal};

pub mod assignment_operator;
pub mod comment;
pub mod key_val;
pub mod operator;

pub trait ASTPrint {
    #[must_use]
    fn ast_print(
        &self,
        depth: usize,
        indentation: &str,
        line_ending: &str,
        should_collapse: bool,
    ) -> String;
}

#[derive(Debug)]
pub struct Document {
    pub statements: Vec<NodeItem>,
}

impl ASTPrint for Document {
    fn ast_print(
        &self,
        depth: usize,
        indentation: &str,
        line_ending: &str,
        should_collapse: bool,
    ) -> String {
        let mut output = String::new();
        for item in &self.statements {
            output.push_str(&item.ast_print(depth, indentation, line_ending, should_collapse));
        }
        output
    }
}

#[derive(Debug)]
pub struct Node {
    pub operator: Option<String>,
    pub identifier: String,
    pub name: Option<String>,
    pub has: Option<String>,
    pub needs: Option<String>,
    pub pass: Option<String>,
    pub index: Option<String>,
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
            self.index.clone().unwrap_or_default(),
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
                        // if self.id_comment.is_some() { " " } else { "" },
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
    len <= 72
}

#[derive(Debug)]
pub enum NodeItem {
    Node(Node),
    Comment(Comment),
    KeyVal(KeyVal),
    EmptyLine,
}
impl ASTPrint for NodeItem {
    fn ast_print(
        &self,
        depth: usize,
        indentation: &str,
        line_ending: &str,
        should_collapse: bool,
    ) -> String {
        match self {
            Self::Node(node) => node.ast_print(depth, indentation, line_ending, should_collapse),
            Self::Comment(comment) => {
                comment.ast_print(depth, indentation, line_ending, should_collapse)
            }
            Self::KeyVal(keyval) => {
                keyval.ast_print(depth, indentation, line_ending, should_collapse)
            }
            Self::EmptyLine => line_ending.to_owned(),
        }
    }
}
