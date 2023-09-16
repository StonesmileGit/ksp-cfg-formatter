use std::fmt::Debug;

use nom::{
    character::complete::{line_ending, multispace0, space0},
    combinator::{map, opt, recognize},
    sequence::{delimited, pair, terminated},
};

use super::{Error, IResult, LocatedSpan, ToRange};

pub(crate) fn ignore_line_ending<'a, F, T>(parser: F) -> impl FnMut(LocatedSpan<'a>) -> IResult<T>
where
    F: FnMut(LocatedSpan<'a>) -> IResult<T>,
{
    terminated(parser, opt(line_ending))
}

pub(crate) fn empty_line(input: LocatedSpan) -> IResult<()> {
    let empty_line = recognize(pair(space0, line_ending));
    map(empty_line, |_| ())(input)
}

/// Evaluate `parser` and wrap the result in a `Some(_)`. Otherwise,
/// emit the  provided `error_msg` and return a `None` while allowing
/// parsing to continue.
pub(crate) fn expect<'a, F, E, T>(
    mut parser: F,
    error_msg: E,
) -> impl FnMut(LocatedSpan<'a>) -> IResult<Option<T>>
where
    F: FnMut(LocatedSpan<'a>) -> IResult<T>,
    E: ToString,
{
    move |input| {
        // dbg!(&input);
        match parser(input) {
            Ok((remaining, out)) => Ok((remaining, Some(out))),
            Err(nom::Err::Error(error) | nom::Err::Failure(error)) => {
                let input = error.input;
                let err = Error(input.to_range(), error_msg.to_string());
                // dbg!(&input);
                // dbg!(&err);
                input.extra.report_error(err); // Push error onto stack.
                Ok((input, None)) // Parsing failed, but keep going.
            }
            Err(err) => Err(err),
        }
    }
}

/// Print debug info
pub(crate) fn debug_fn<'a, F, E, T>(
    mut parser: F,
    success_msg: E,
    print: bool,
) -> impl FnMut(LocatedSpan<'a>) -> IResult<T>
where
    F: FnMut(LocatedSpan<'a>) -> IResult<T>,
    E: ToString,
    T: Debug,
{
    move |input| {
        // dbg!(&input);
        let print = print & false;
        match parser(input) {
            Ok((remaining, out)) => {
                if print {
                    println!(
                        "{}: {:#?}\nRemaining:\n{}\n",
                        success_msg.to_string(),
                        &out,
                        &remaining // ""
                    );
                }
                Ok((remaining, out))
            }
            Err(nom::Err::Error(error) | nom::Err::Failure(error)) => {
                if print {
                    println!("{}: {:#?}\n", success_msg.to_string(), &error);
                }
                Err(nom::Err::Error(error))
            }
            Err(err) => Err(err),
        }
    }
}

/// A combinator that takes a parser `inner` and produces a parser that also consumes
/// trailing whitespace, returning the output of `inner`.
pub(crate) fn ws<'a, F, O>(inner: F) -> impl FnMut(LocatedSpan<'a>) -> IResult<O>
where
    F: FnMut(LocatedSpan<'a>) -> IResult<O>,
{
    delimited(space0, inner, space0)
}

/// A combinator that takes a parser `inner` and produces a parser that also consumes both leading and
/// trailing whitespace, returning the output of `inner`.
pub(crate) fn ws_le<'a, F, O>(inner: F) -> impl FnMut(LocatedSpan<'a>) -> IResult<O>
where
    F: FnMut(LocatedSpan<'a>) -> IResult<O>,
{
    delimited(multispace0, inner, multispace0)
}
