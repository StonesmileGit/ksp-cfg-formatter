use std::fmt::Display;

pub trait ASTPrint {
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
            output.push_str(
                item.ast_print(depth, indentation, line_ending, should_collapse)
                    .as_str(),
            );
        }
        output
    }
}

#[derive(Debug)]
pub struct Node {
    pub identifier: String,
    pub id_comment: Option<Comment>,
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
        let indentation_str = indentation.repeat(depth);
        match self.block.len() {
            0 if self.id_comment.is_none() => {
                format!(
                    "{}{} {{}}{}{}",
                    indentation_str,
                    self.identifier,
                    self.trailing_comment.as_ref().unwrap_or(&Comment {
                        text: String::new()
                    }),
                    line_ending
                )
            }
            1 if should_collapse && short_node(self) => {
                format!(
                    "{}{} {{ {} }}{}{}",
                    indentation_str,
                    self.identifier,
                    self.block
                        .first()
                        .unwrap()
                        .ast_print(0, indentation, "", should_collapse),
                    self.trailing_comment.as_ref().unwrap_or(&Comment {
                        text: String::new()
                    }),
                    line_ending
                )
            }
            _ => {
                let mut output = format!(
                    "{}{}{}{}{}{}{{{}",
                    indentation_str,
                    self.identifier,
                    if self.id_comment.is_some() { " " } else { "" },
                    self.id_comment.as_ref().unwrap_or(&Comment {
                        text: String::new()
                    }),
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
                        .to_string()
                        .as_str(),
                );
                output.push_str(line_ending);
                output
            }
        }
    }
}

fn short_node(arg: &Node) -> bool {
    if arg.id_comment.is_some() {
        return false;
    }
    let mut len = 7; // Include the opening/closing bracket and spaces around operator
    len += arg.identifier.chars().count();
    match arg.block.first().unwrap() {
        NodeItem::KeyVal(kv) => {
            len += kv.key.chars().count();
            len += kv.operator.chars().count();
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

#[derive(Debug)]
pub struct Comment {
    pub text: String,
}

impl Display for Comment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text)
    }
}

impl ASTPrint for Comment {
    fn ast_print(&self, depth: usize, indentation: &str, line_ending: &str, _: bool) -> String {
        let indentation = indentation.repeat(depth);
        format!("{}{}{}", indentation, self.text, line_ending)
    }
}

#[derive(Debug)]
pub struct KeyVal {
    pub key: String,
    pub operator: String,
    pub val: String,
    pub comment: Option<Comment>,
}

impl ASTPrint for KeyVal {
    fn ast_print(&self, depth: usize, indentation: &str, line_ending: &str, _: bool) -> String {
        let indentation = indentation.repeat(depth);
        format!(
            "{}{} {} {}{}{}",
            indentation,
            self.key,
            self.operator,
            self.val,
            self.comment.as_ref().unwrap_or(&Comment {
                text: String::new()
            }),
            line_ending
        )
    }
}
