use std::num::ParseIntError;

use pest::iterators::Pair;

use crate::reader::Rule;

use super::{
    comment::Comment,
    has::{HasBlock, HasBlockError},
    indices::Index,
    key_val::{KeyVal, KeyValError},
    operator::{Operator, OperatorParseError},
    path::Path,
    ASTPrint, NodeItem,
};

#[derive(Debug, Default)]
pub struct Node<'a> {
    pub path: Option<Path<'a>>,
    pub operator: Option<Operator>,
    pub identifier: String,
    pub name: Option<String>,
    pub has: Option<HasBlock<'a>>,
    pub needs: Option<String>,
    pub pass: Option<String>,
    pub index: Option<Index>,
    pub id_comment: Option<Comment>,
    pub comments_after_newline: Vec<Comment>,
    pub block: Vec<NodeItem<'a>>,
    pub trailing_comment: Option<Comment>,
}

pub fn parse_block_items<'a>(pair: Pair<'a, Rule>) -> Result<Vec<NodeItem>, NodeParseError<'a>> {
    assert!(matches!(pair.as_rule(), Rule::nodeBody | Rule::document));
    // if matches!(pair.as_rule(), Rule::nodeBody) {
    //     dbg!(&pair);
    // }
    let mut block_items = vec![];
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::node => block_items.push(Ok(NodeItem::Node(Node::try_from(pair)?))),
            Rule::Comment => block_items.push(Ok(NodeItem::Comment(
                Comment::try_from(pair).expect("Parsing a comment is Infallable"),
            ))),
            Rule::assignment => block_items.push(Ok(NodeItem::KeyVal(KeyVal::try_from(pair)?))),
            Rule::EmptyLine => block_items.push(Ok(NodeItem::EmptyLine)),
            // Rule::closingbracket => break,
            Rule::EOI | Rule::Newline => (),
            // _ => panic!("abc: {:?}", pair),
            _ => unreachable!(),
        }
    }
    block_items.into_iter().collect()
}

pub enum NodeParseError<'a> {
    HasBlockError(HasBlockError),
    ParseIntError(ParseIntError),
    OperatorParseError(OperatorParseError<'a>),
    KeyValError(KeyValError<'a>),
}

impl<'a> From<KeyValError<'a>> for NodeParseError<'a> {
    fn from(value: KeyValError<'a>) -> Self {
        Self::KeyValError(value)
    }
}

impl<'a> From<OperatorParseError<'a>> for NodeParseError<'a> {
    fn from(value: OperatorParseError<'a>) -> Self {
        Self::OperatorParseError(value)
    }
}

impl<'a> From<ParseIntError> for NodeParseError<'a> {
    fn from(value: ParseIntError) -> Self {
        Self::ParseIntError(value)
    }
}

impl<'a> From<HasBlockError> for NodeParseError<'a> {
    fn from(value: HasBlockError) -> Self {
        Self::HasBlockError(value)
    }
}

impl<'a> TryFrom<Pair<'a, Rule>> for Node<'a> {
    type Error = NodeParseError<'a>;

    fn try_from(rule: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        assert!(matches!(rule.as_rule(), Rule::node));
        let mut pairs = rule.clone().into_inner();

        let mut node = Node::default();

        let mut body_seen = false;
        let mut newline_seen = false;

        for pair in pairs.by_ref() {
            match pair.as_rule() {
                Rule::Comment => {
                    if body_seen {
                        node.trailing_comment =
                            Some(Comment::try_from(pair).expect("Parsing a comment is Infallable"));
                    } else {
                        if newline_seen {
                            node.comments_after_newline.push(
                                Comment::try_from(pair).expect("Parsing a comment is Infallable"),
                            );
                        } else {
                            node.id_comment = Some(
                                Comment::try_from(pair).expect("Parsing a comment is Infallable"),
                            );
                        }
                    }
                }
                Rule::openingbracket => (),
                Rule::closingbracket => (),
                Rule::Newline => newline_seen = true,

                Rule::identifier => node.identifier = pair.as_str().to_string(),
                Rule::nameBlock => node.name = Some(pair.as_str().to_string()),
                Rule::hasBlock => node.has = Some(HasBlock::try_from(pair)?),
                Rule::needsBlock => node.needs = Some(pair.as_str().to_string()),
                Rule::passBlock => node.pass = Some(pair.as_str().to_string()),
                Rule::index => node.index = Some(super::indices::Index::try_from(pair)?),
                Rule::operator => node.operator = Some(Operator::try_from(pair)?),
                Rule::path => {
                    node.path =
                        Some(Path::try_from(pair).expect("Parsing path is supposedly Infallable"))
                }
                Rule::nodeBody => {
                    node.block = parse_block_items(pair)?;
                    body_seen = true;
                }
                // _ => panic!("{}", pair),
                _ => unreachable!(),
            }
        }
        Ok(node)
    }
}

impl<'a> ASTPrint for Node<'a> {
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
            "{}{}{}{}{}{}{}{}{}",
            if self.path.is_some() { "#" } else { "" },
            self.path.clone().map_or("".to_owned(), |p| p.to_string()),
            self.operator.clone().unwrap_or_default(),
            self.identifier,
            self.name.clone().unwrap_or_default(),
            self.has.clone().unwrap_or_default(),
            self.pass.clone().unwrap_or_default(),
            self.needs.clone().unwrap_or_default(),
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
