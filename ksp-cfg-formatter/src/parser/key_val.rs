use super::{
    parser_helpers::{debug_fn, ignore_line_ending, range_wrap, ws},
    ASTPrint, ArrayIndex, AssignmentOperator, Comment, Index, NeedsBlock, Operator, Path, Range,
    Ranged, {ASTParse, IResult, LocatedSpan},
};
use nom::{
    branch::alt,
    bytes::complete::{is_a, tag},
    character::complete::{anychar, char, line_ending, none_of, one_of, space0, space1},
    combinator::{all_consuming, eof, map, opt, peek, recognize},
    multi::{many1, many_till, separated_list1},
    sequence::{pair, preceded, terminated, tuple},
};
use nom_unicode::complete::alphanumeric1;

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
    key_padding: Option<usize>,
    /// The assignment operator between the variable and the value
    pub assignment_operator: Ranged<AssignmentOperator>,
    /// The value to use in the assignment
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
                .map_or_else(String::new, std::string::ToString::to_string),
            self.array_index
                .as_deref()
                .map_or_else(String::new, std::string::ToString::to_string),
        )
    }
    pub(crate) fn set_key_padding(&mut self, n: usize) {
        self.key_padding = Some(n - self.left_side().len());
    }
}

impl<'a> ASTPrint for KeyVal<'a> {
    fn ast_print(
        &self,
        depth: usize,
        indentation: &str,
        line_ending: &str,
        _: Option<bool>,
    ) -> String {
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
                .map_or_else(String::new, std::string::ToString::to_string),
            self.array_index
                .as_deref()
                .map_or_else(String::new, std::string::ToString::to_string),
            self.key_padding.map_or_else(String::new, |p| " ".repeat(p)),
            self.assignment_operator,
            self.val,
            self.comment.as_ref().map_or("", |c| c.text),
            line_ending
        )
    }
}

impl<'a> ASTParse<'a> for KeyVal<'a> {
    fn parse(input: LocatedSpan<'a>) -> IResult<Ranged<KeyVal<'a>>> {
        let parser = move |input| {
            // This parses anything that could potentially be a key
            let (input, dumb_key) = recognize(many_till(
                anychar,
                peek(alt((
                    recognize(preceded(space0, AssignmentOperator::parse)),
                    recognize(Comment::parse),
                    recognize(one_of("{}\n\r")),
                ))),
            ))(input)?;
            let (complete_key, errors) = proper_key_parser(dumb_key);

            let (input, assignment_operator) = ws(AssignmentOperator::parse)(input)?;

            let (input, (value, comment)) = map(
                ignore_line_ending(pair(
                    range_wrap(recognize(many_till(
                        anychar,
                        peek(alt((
                            recognize(Comment::parse),
                            preceded(space0, is_a("}\r\n")),
                        ))),
                    ))),
                    terminated(
                        opt(Comment::parse),
                        opt(terminated(space0, peek(line_ending))),
                    ),
                )),
                |(s, c)| (s.map(|s| *s.fragment()), c),
            )(input)?;

            // let (input, comment) = opt(ignore_line_ending(Comment::parse))(input)?;

            let key_val = KeyVal {
                path: complete_key.0,
                operator: complete_key.1,
                key: complete_key.2,
                needs: complete_key.3,
                index: complete_key.4,
                array_index: complete_key.5,
                key_padding: None,
                assignment_operator,
                val: value,
                comment,
            };
            for err in errors {
                input.extra.report_error(err);
            }
            Ok((input, key_val))
        };
        range_wrap(debug_fn(parser, "keyVal", true))(input)
    }
}

type ParsedKey<'a> = (
    Option<Ranged<Path<'a>>>,
    Option<Ranged<Operator>>,
    Ranged<&'a str>,
    Option<Ranged<NeedsBlock<'a>>>,
    Option<Ranged<Index>>,
    Option<Ranged<ArrayIndex>>,
);

fn proper_key_parser(input: LocatedSpan<'_>) -> (ParsedKey, Vec<super::Error>) {
    // Clear errors on dumb_key to avoid duplicated errors
    input.extra.errors.borrow_mut().clear();

    let mut res_tuple = (
        None,
        None,
        Ranged::new("", Range::default()),
        None,
        None,
        None,
    );

    let (input, path) = opt(preceded(char('*'), Path::parse))(input).expect("opt should not fail?");
    res_tuple.0 = path;
    let (input, operator) = opt(Operator::parse)(input).expect("opt should not fail?");
    res_tuple.1 = operator;

    let key = range_wrap(map(
        recognize(separated_list1(
            space1::<LocatedSpan, _>,
            recognize(many1(alt((
                alphanumeric1,
                is_a("#_.?()"),
                recognize(terminated(
                    one_of("-+*"),
                    alt((recognize(none_of("=")), eof)),
                )),
                terminated(tag("/"), none_of("/=")),
            )))),
        )),
        |s| *s.fragment(),
    ));
    let needs = opt(NeedsBlock::parse);
    let index = opt(Index::parse);
    let array_index = opt(ArrayIndex::parse);

    // TODO: Where can Needs be located? Index *HAS* to be before array index
    let proper_key = tuple((key, needs, index, array_index));
    // Everything in the dumb key has to be parsed, otherwise there is an error in the text
    let res = all_consuming(proper_key)(input.clone());
    match res {
        Ok((rest, proper_key_tuple)) => {
            res_tuple.2 = proper_key_tuple.0;
            res_tuple.3 = proper_key_tuple.1;
            res_tuple.4 = proper_key_tuple.2;
            res_tuple.5 = proper_key_tuple.3;
            (res_tuple, rest.extra.errors.borrow_mut().clone())
        }
        // If an error is encountered, just stuff the pseudo-key inside the key, and report the error
        Err(nom::Err::Error(error) | nom::Err::Failure(error)) => {
            res_tuple.2 = Ranged::from(input);
            (
                res_tuple,
                vec![super::Error {
                    message: format!(
                        "failed to parse key. Unexpected `{}`",
                        error.input.fragment()
                    ),
                    source: (*error.input.fragment()).to_string(),
                    range: Range::from(error.input),
                    severity: super::Severity::Error,
                    context: None,
                }],
            )
        }
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {

    use crate::parser::{LocatedSpan, State};

    use super::*;
    #[test]
    fn test_key_val() {
        let input = "key = val\r\n";
        let res = KeyVal::parse(LocatedSpan::new_extra(input, State::default()));

        match res {
            Ok(it) => assert_eq!(input, it.1.ast_print(0, "\t", "\r\n", None)),
            Err(err) => panic!("{}", err),
        }
    }
    #[test]
    fn test_key_val_2() {
        let input = "*@PART[RO-M55]/deleteMe = true\r\n";
        let res = KeyVal::parse(LocatedSpan::new_extra(input, State::default()));

        match res {
            Ok(it) => assert_eq!(input, it.1.ast_print(0, "\t", "\r\n", None)),
            Err(err) => panic!("{}", err),
        }
    }

    #[test]
    fn test_key_val_error() {
        let input = "deleteMe[-1] = true\r\n";
        let res = KeyVal::parse(LocatedSpan::new_extra(input, State::default()));

        match res {
            Ok(it) => {
                assert_eq!(
                    it.0.extra.errors.into_inner()[0].message,
                    "failed to parse key. Unexpected `-1]`"
                );
                assert_eq!(input, it.1.ast_print(0, "\t", "\r\n", None))
            }
            Err(err) => panic!("{}", err),
        }
    }
}
