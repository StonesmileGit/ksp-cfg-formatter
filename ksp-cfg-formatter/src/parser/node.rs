use itertools::Itertools;
use nom::branch::alt;
use nom::bytes::complete::{is_a, is_not};
use nom::character::complete::{
    anychar, char, line_ending, multispace0, multispace1, one_of, space0,
};
use nom::combinator::{all_consuming, map, opt, peek, recognize};
use nom::multi::{many0, many1, many_till, separated_list0};
use nom::sequence::{delimited, preceded, tuple};
use nom_unicode::complete::alphanumeric1;

use super::nom::utils::{
    debug_fn, empty_line, error_till, expect, expect_context, get_range, non_empty, range_wrap, ws,
    ws_le,
};
use super::Ranged;
use super::{
    nom::CSTParse, ASTPrint, Comment, HasBlock, Index, KeyVal, NeedsBlock, NodeItem, Operator,
    Pass, Path, Range,
};

/// A node in the config file. Both top level node and internal node
#[derive(Debug, Default, Clone)]
pub struct Node<'a> {
    top_level: bool,
    /// Optional path to node, only allowed on internal nodes
    pub path: Option<Ranged<Path<'a>>>,
    /// Optional operator
    pub operator: Option<Ranged<Operator>>,
    /// Identifier of the node
    pub identifier: Ranged<&'a str>,
    /// Optional name of the node. Same as `:HAS[name[<name>]]`
    pub name: Option<Ranged<Vec<&'a str>>>,
    /// Optional HAS block
    pub has: Option<Ranged<HasBlock<'a>>>,
    /// Optional NEEDS block
    pub needs: Option<Ranged<NeedsBlock<'a>>>,
    /// Pass for the patch to run
    pub pass: Option<Ranged<Pass<'a>>>,
    /// Optional index of the node to match
    pub index: Option<Ranged<Index>>,
    /// Optional comment after the identifier
    pub id_comment: Option<Ranged<Comment<'a>>>,
    /// optional comments between identifier line and opening bracket
    pub comments_after_newline: Vec<Ranged<Comment<'a>>>,
    /// Items inside the node
    pub block: Vec<NodeItem<'a>>,
    /// Optional trailing comment after the closing bracket
    pub trailing_comment: Option<Ranged<Comment<'a>>>,
}

impl<'a> Node<'a> {
    /// Indicates if a node is a top-level node or not
    #[must_use]
    pub const fn top_level(&self) -> bool {
        self.top_level
    }
    /// Returns an iterator over all of the Nodes contained within this node
    pub fn iter_nodes(&self) -> impl Iterator<Item = &Ranged<Node>> {
        self.block.iter().filter_map(|n| {
            if let NodeItem::Node(node) = n {
                Some(node)
            } else {
                None
            }
        })
    }
    /// Returns an iterator over all of the Assignments contained within this node
    pub fn iter_keyvals(&self) -> impl Iterator<Item = &Ranged<KeyVal>> {
        self.block.iter().filter_map(|n| {
            if let NodeItem::KeyVal(kv) = n {
                Some(kv)
            } else {
                None
            }
        })
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
            self.path.clone().map_or(String::new(), |p| p.to_string()),
            self.operator.clone().unwrap_or_default(),
            self.identifier,
            self.name.clone().map_or(String::new(), |name| format!(
                "[{}]",
                name.iter().format("|")
            )),
            self.has.clone().unwrap_or_default(),
            self.pass.clone().map_or(String::new(), |p| p.to_string()),
            self.needs.clone().map_or(String::new(), |n| n.to_string()),
            self.index
                .as_deref()
                .map_or(String::new(), std::string::ToString::to_string),
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
                            .map_or_else(|| "", |c| c.text),
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
                            .map_or_else(|| "", |c| c.text),
                        line_ending
                    )
                }
                _ => {
                    let mut output = format!(
                        "{}{}{}{}{}{{{}",
                        indentation_str,
                        complete_node_name,
                        self.id_comment.as_ref().map_or_else(|| "", |c| c.text),
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
                            .map_or_else(|| "", |c| c.text),
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

// TODO: replace with just fetching the Range of the node
// Doesn't work. The node could be multi line before parsing, and the ast_print function isn't available since that is recursion
fn short_node(arg: &Node) -> bool {
    const MAX_LENGTH: usize = 72;
    if arg.id_comment.is_some() {
        return false;
    }
    let mut len = 7; // Include the opening/closing bracket and spaces around operator
    len += arg
        .path
        .clone()
        .map_or(0, |path| path.to_string().chars().count());
    len += arg
        .operator
        .clone()
        .map_or(0, |op| op.to_string().chars().count());
    len += arg.identifier.chars().count();
    len += arg
        .name
        .clone()
        .map_or(0, |name| name.iter().map(|e| e.chars().count()).sum());
    len += arg
        .has
        .clone()
        .map_or(0, |has| has.to_string().chars().count());
    len += arg
        .needs
        .clone()
        .map_or(0, |needs| needs.to_string().chars().count());
    len += arg
        .pass
        .clone()
        .map_or(0, |p| p.to_string().chars().count());
    len += arg
        .index
        .as_deref()
        .map_or(0, |id| id.to_string().chars().count());

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

use super::nom::{utils::ignore_line_ending, IResult, LocatedSpan};

impl<'a> CSTParse<'a, Ranged<Node<'a>>> for Node<'a> {
    fn parse(input: LocatedSpan<'a>) -> IResult<Ranged<Node<'a>>> {
        let top_level = input.extra.state.top_level;

        // TODO: make sure this doesn't match too much
        let dumb_identifier = recognize(tuple((
            many_till(
                anychar,
                peek(alt((
                    recognize(Comment::parse),
                    recognize(preceded(multispace0, one_of("{}\r\n"))),
                ))),
            ),
            many0(alt((recognize(Comment::parse), recognize(multispace1)))),
        )));

        let block = preceded(opt(line_ending), preceded(space0, parse_block));

        let trailing_comment = opt(Comment::parse);

        let mut node = tuple((dumb_identifier, block, trailing_comment));
        range_wrap(move |input| {
            let (rest, pseudo_node) = node(input)?;
            let (dumb_identifier, block, trailing_comment) = pseudo_node;

            let (complete_identifier, errors) = dumb_identifier_parser(dumb_identifier);
            let node = Node {
                top_level,
                path: complete_identifier.0,
                operator: complete_identifier.1,
                identifier: complete_identifier.2,
                name: complete_identifier.3,
                has: complete_identifier.4,
                needs: complete_identifier.5,
                pass: complete_identifier.6,
                index: complete_identifier.7,
                id_comment: complete_identifier.8,
                comments_after_newline: complete_identifier.9,
                block,
                trailing_comment,
            };
            for err in errors {
                rest.extra.report_error(err);
            }
            Ok((rest, node))
        })(input)
    }
}

enum HasPassNeedsIndex<'a> {
    Has(Ranged<HasBlock<'a>>),
    Pass(Ranged<Pass<'a>>),
    Needs(Ranged<NeedsBlock<'a>>),
    Index(Ranged<Index>),
}

fn dumb_identifier_parser(
    dumb_identifier: LocatedSpan,
) -> (ParsedIdentifier, Vec<super::nom::Error>) {
    // Clear errors on dumb_key to avoid duplicated errors
    dumb_identifier.extra.errors.borrow_mut().clear();

    let path = opt(preceded(char('#'), Path::parse));
    let operator = opt(Operator::parse);
    // identifier = ${ ("-" | "_" | "." | "+" | "*" | "?" | LETTER | ASCII_DIGIT)+ }
    let identifier = range_wrap(map(
        recognize(many1(alt((
            alphanumeric1::<LocatedSpan, _>,
            is_a("-_.+*?"),
        )))),
        |inner| *inner.fragment(),
    ));
    let name = opt(parse_name);
    let has = HasBlock::parse;
    let needs = NeedsBlock::parse;
    let pass = Pass::parse;
    let index = Index::parse;
    let id_comment = opt(Comment::parse);
    let comments_after_newline = many0(preceded(opt(line_ending), Comment::parse));
    let complete_identifier = tuple((
        debug_fn(path, "Got path", true),
        debug_fn(operator, "Got operator", true),
        debug_fn(identifier, "Got node id", true),
        debug_fn(name, "Got name", true),
        many0(alt((
            map(has, HasPassNeedsIndex::Has),
            map(pass, HasPassNeedsIndex::Pass),
            map(needs, HasPassNeedsIndex::Needs),
            map(index, HasPassNeedsIndex::Index),
        ))),
        debug_fn(id_comment, "Got id_comment", true),
        debug_fn(comments_after_newline, "Got comments after newline", true),
    ));
    let res = all_consuming(ws_le(complete_identifier))(dumb_identifier.clone());
    match res {
        Ok((rest, proper_identifier_tuple)) => (
            map_correct_identifier(&rest, proper_identifier_tuple),
            rest.extra.errors.borrow_mut().clone(),
        ),
        // If an error is encountered, just stuff the pseudo-identifier inside the identifier, and report the error
        Err(nom::Err::Error(error) | nom::Err::Failure(error)) => (
            (
                None,
                None,
                Ranged::new(dumb_identifier.fragment(), Range::from(dumb_identifier)),
                None,
                None,
                None,
                None,
                None,
                None,
                vec![],
            ),
            vec![super::nom::Error {
                message: format!(
                    "failed to parse identifier. Unexpected `{}`",
                    error.input.fragment()
                ),
                source: (*error.input.fragment()).to_string(),
                range: Range::from(error.input),
                severity: super::nom::Severity::Error,
                context: None,
            }],
        ),
        _ => unreachable!(),
    }
}

type ParsedIdentifier<'a> = (
    Option<Ranged<Path<'a>>>,
    Option<Ranged<Operator>>,
    Ranged<&'a str>,
    Option<Ranged<Vec<&'a str>>>,
    Option<Ranged<HasBlock<'a>>>,
    Option<Ranged<NeedsBlock<'a>>>,
    Option<Ranged<Pass<'a>>>,
    Option<Ranged<Index>>,
    Option<Ranged<Comment<'a>>>,
    Vec<Ranged<Comment<'a>>>,
);

type ToBeParsedIdentifier<'a> = (
    Option<Ranged<Path<'a>>>,
    Option<Ranged<Operator>>,
    Ranged<&'a str>,
    Option<Ranged<Vec<&'a str>>>,
    Vec<HasPassNeedsIndex<'a>>,
    Option<Ranged<Comment<'a>>>,
    Vec<Ranged<Comment<'a>>>,
);

type ComboTupleFiltered<'a> = (
    Vec<Ranged<HasBlock<'a>>>,
    Vec<Ranged<Pass<'a>>>,
    Vec<Ranged<NeedsBlock<'a>>>,
    Vec<Ranged<Index>>,
);

fn split_combo(combo_list: Vec<HasPassNeedsIndex>) -> ComboTupleFiltered {
    let mut res = (vec![], vec![], vec![], vec![]);
    for combo in combo_list {
        match combo {
            HasPassNeedsIndex::Has(has) => res.0.push(has),
            HasPassNeedsIndex::Pass(pass) => res.1.push(pass),
            HasPassNeedsIndex::Needs(needs) => res.2.push(needs),
            HasPassNeedsIndex::Index(index) => res.3.push(index),
        }
    }
    res
}

fn map_correct_identifier<'a>(
    rest: &LocatedSpan<'a>,
    input_tuple: ToBeParsedIdentifier<'a>,
) -> ParsedIdentifier<'a> {
    let (has_vec, pass_vec, needs_vec, index_vec) = split_combo(input_tuple.4);
    if has_vec.len() > 1 {
        for has in &has_vec[1..] {
            rest.extra.report_error(super::nom::Error {
                message: "Got extra HAS block".to_owned(),
                range: has.range,
                source: has.to_string(),
                severity: super::nom::Severity::Error,
                context: None,
            });
        }
    }
    let has = has_vec.first().cloned();

    if needs_vec.len() > 1 {
        for needs in &needs_vec[1..] {
            rest.extra.report_error(super::nom::Error {
                message: "Got extra NEEDS block".to_owned(),
                range: needs.range,
                source: needs.to_string(),
                severity: super::nom::Severity::Error,
                context: None,
            });
        }
    }
    let needs = needs_vec.first().cloned();

    if pass_vec.len() > 1 {
        for pass in &pass_vec[1..] {
            rest.extra.report_error(super::nom::Error {
                message: "Got extra PASS block".to_owned(),
                range: pass.range,
                source: pass.to_string(),
                severity: super::nom::Severity::Error,
                context: None,
            });
        }
    }
    let pass = pass_vec.first().cloned();

    let index = index_vec.first().cloned();
    (
        input_tuple.0,
        input_tuple.1,
        input_tuple.2,
        input_tuple.3,
        has,
        needs,
        pass,
        index,
        input_tuple.5,
        input_tuple.6,
    )
}

fn parse_name(input: LocatedSpan) -> IResult<Ranged<Vec<&str>>> {
    let parser = |input| {
        let (input, (_, context_range)) = get_range(char('['))(input)?;
        // TODO: Tag both brackets as a warning to make it more visible
        let (input, res) = separated_list0(char('|'), is_not("|]"))(input)?;
        let (input, _) = expect_context(
            char(']'),
            "Expected closing `]`",
            Ranged {
                inner: "Expected due to `[` found here".to_string(),
                range: context_range,
            },
        )(input)?;
        let names = res.iter().map(|e: &LocatedSpan| *e.fragment()).collect();
        Ok((input, names))
    };
    range_wrap(parser)(input)
}

/// Takes a parser and sets the settings according to what is needed for parsing an inner block, and then setting them back as needed on the returned settings as needed
pub(crate) fn settings_for_inner_block<'a, F, T>(
    mut parser: F,
) -> impl FnMut(LocatedSpan<'a>) -> IResult<T>
where
    F: FnMut(LocatedSpan<'a>) -> IResult<T>,
{
    move |input: LocatedSpan<'a>| {
        let top_level = input.extra.state.top_level;
        let mut input = input;
        input.extra.state.top_level = false;
        let res = parser(input);
        match res {
            Ok(mut it) => {
                it.0.extra.state.top_level = top_level;
                Ok(it)
            }
            Err(nom::Err::Error(mut err) | nom::Err::Failure(mut err)) => {
                err.input.extra.state.top_level = top_level;
                Err(nom::Err::Error(err))
            }
            Err(err) => Err(err),
        }
    }
}

fn parse_block(input: LocatedSpan) -> IResult<Vec<NodeItem>> {
    let block = delimited(
        char('{'),
        ws_le(many0(ws(alt((
            map(ignore_line_ending(ws(Comment::parse)), |c| {
                NodeItem::Comment(c)
            }),
            map(ws(empty_line), |_| NodeItem::EmptyLine),
            map(ws(KeyVal::parse), NodeItem::KeyVal),
            settings_for_inner_block(map(ignore_line_ending(ws(Node::parse)), NodeItem::Node)),
            debug_fn(
                map(recognize(error_till(non_empty(is_not("}\r\n")))), |s| {
                    NodeItem::Error(Ranged::new(s.clone().fragment(), s.into()))
                }),
                "Got an error while parsing node. Skipped line",
                true,
            ),
        ))))),
        expect(char('}'), "Expected closing }"),
    );
    map(block, |inner: Vec<NodeItem>| inner)(input)
}

#[cfg(test)]
mod tests {

    use crate::parser::nom::{LocatedSpan, State};

    use super::*;
    #[test]
    fn test_node() {
        let input = "node { key = val }\r\n";
        let res = Node::parse(LocatedSpan::new_extra(input, State::default()));

        match res {
            Ok(it) => assert_eq!(input, it.1.ast_print(0, "\t", "\r\n", true)),
            Err(err) => panic!("{}", err),
        }
    }
    #[test]
    fn test_node_2() {
        let input = "node\r\n{\r\n\tkey = val\r\n\tkey = val\r\n}\r\n";
        let res = Node::parse(LocatedSpan::new_extra(input, State::default()));

        match res {
            Ok(it) => assert_eq!(input, it.1.ast_print(0, "\t", "\r\n", true)),
            Err(err) => panic!("{:#?}", err),
        }
    }
}
