use std::fmt::Debug;

use log::{debug, trace};
use nom::{
    branch::alt,
    character::complete::{line_ending, multispace0, space0},
    combinator::{eof, map, opt, recognize, verify},
    error::ErrorKind,
    sequence::{delimited, pair, terminated},
    Slice,
};

use crate::parser::{Position, Range, Ranged};

use super::{Error, IResult, LocatedSpan};

pub(crate) fn ignore_line_ending<'a, F, T>(parser: F) -> impl FnMut(LocatedSpan<'a>) -> IResult<T>
where
    F: FnMut(LocatedSpan<'a>) -> IResult<T>,
{
    terminated(parser, opt(line_ending))
}

pub(crate) fn empty_line(input: LocatedSpan) -> IResult<()> {
    let empty_line = recognize(pair(space0, alt((line_ending, eof))));
    map(empty_line, |_| ())(input)
}

/// Make sure the inner parser matched at least one char from the input
pub(crate) fn non_empty<'a, F>(
    parser: F,
) -> impl FnMut(LocatedSpan<'a>) -> IResult<'a, LocatedSpan<'a>>
where
    F: FnMut(LocatedSpan<'a>) -> IResult<LocatedSpan<'a>>,
{
    verify(parser, |s| !s.is_empty())
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
        match parser(input) {
            Ok((remaining, out)) => Ok((remaining, Some(out))),
            Err(nom::Err::Error(error) | nom::Err::Failure(error)) => {
                let input = error.input;
                let length = usize::from(!input.is_empty());
                let err = Error {
                    source: (*input.fragment()).to_string(),
                    range: Range::from(input.slice(0..length)),
                    message: error_msg.to_string(),
                    severity: super::Severity::Error,
                    context: None,
                };
                input.extra.report_error(err); // Push error onto stack.
                Ok((input, None)) // Parsing failed, but keep going.
            }
            Err(err) => Err(err),
        }
    }
}
/// Evaluate `parser` and wrap the result in a `Some(_)`. Otherwise,
/// emit the  provided `error_msg` and return a `None` while allowing
/// parsing to continue.
// pub(crate) fn expect_warning<'a, F, E, T>(
//     mut parser: F,
//     error_msg: E,
// ) -> impl FnMut(LocatedSpan<'a>) -> IResult<Option<T>>
// where
//     F: FnMut(LocatedSpan<'a>) -> IResult<T>,
//     E: ToString,
// {
//     move |input| {
//         match parser(input) {
//             Ok((remaining, out)) => Ok((remaining, Some(out))),
//             Err(nom::Err::Error(error) | nom::Err::Failure(error)) => {
//                 let input = error.input;
//                 let length = usize::from(!input.is_empty());
//                 let err = Error {
//                     source: (*input.fragment()).to_string(),
//                     range: Range::from(input.slice(0..length)),
//                     message: error_msg.to_string(),
//                     severity: super::Severity::Warning,
//                     context: None,
//                 };
//                 input.extra.report_error(err); // Push error onto stack.
//                 Ok((input, None)) // Parsing failed, but keep going.
//             }
//             Err(err) => Err(err),
//         }
//     }
// }

/// Evaluate `parser` and wrap the result in a `Some(_)`. Otherwise,
/// emit the  provided `error_msg` and return a `None` while allowing
/// parsing to continue.
pub(crate) fn expect_context<'a, F, E, T>(
    mut parser: F,
    error_msg: E,
    context_msg: Ranged<String>,
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
                let length = usize::from(!input.is_empty());
                let err = Error {
                    source: (*input.fragment()).to_string(),
                    range: Range::from(input.slice(0..length)),
                    message: error_msg.to_string(),
                    severity: super::Severity::Error,
                    context: Some(context_msg.clone()),
                };
                // dbg!(&input);
                // dbg!(&err);
                input.extra.report_error(err); // Push error onto stack.
                Ok((input, None)) // Parsing failed, but keep going.
            }
            Err(err) => Err(err),
        }
    }
}

pub(crate) fn range_wrap<'a, F, T>(
    mut parser: F,
) -> impl FnMut(LocatedSpan<'a>) -> IResult<Ranged<T>>
where
    F: FnMut(LocatedSpan<'a>) -> IResult<T>,
{
    move |input| {
        let start = Position::from_located_span(&input);
        let (rest, out) = parser(input)?;
        let end = Position::from_located_span(&rest);
        Ok((rest, Ranged::new(out, Range { start, end })))
    }
}

pub(crate) fn get_range<'a, F, T>(
    mut parser: F,
) -> impl FnMut(LocatedSpan<'a>) -> IResult<(T, Range)>
where
    F: FnMut(LocatedSpan<'a>) -> IResult<T>,
{
    move |input| {
        let start = Position::from_located_span(&input);
        let (rest, out) = match parser(input) {
            Ok(it) => it,
            Err(err) => return Err(err),
        };
        let end = Position::from_located_span(&rest);
        Ok((rest, (out, Range { start, end })))
    }
}

pub(crate) fn error_till<'a, F>(mut parser: F) -> impl FnMut(LocatedSpan<'a>) -> IResult<'a, ()>
where
    F: FnMut(LocatedSpan<'a>) -> IResult<LocatedSpan>,
{
    move |input| match parser(input.clone()) {
        Ok((rem, out)) => {
            if out.len() > 0 {
                rem.extra.report_error(Error {
                    source: (*out.fragment()).to_string(),
                    message: format!("unexpected `{}`", out.fragment()),
                    range: Range::from(out),
                    severity: super::Severity::Error,
                    context: None,
                });
                Ok((rem, ()))
            } else {
                Err(nom::Err::Error(nom::error::Error {
                    input,
                    code: ErrorKind::Fail,
                }))
            }
        }
        Err(err) => Err(err),
    }
}

/// Print debug info
pub(crate) fn debug_fn<'a, F, E, T>(
    mut parser: F,
    debug_msg: E,
    print: bool,
) -> impl FnMut(LocatedSpan<'a>) -> IResult<T>
where
    F: FnMut(LocatedSpan<'a>) -> IResult<T>,
    E: ToString,
    T: Debug,
{
    move |input| {
        trace!("{}", &input);
        match parser(input) {
            Ok((remaining, out)) => {
                if print {
                    debug!(
                        "Ok branch: {}: {:#?}\nRemaining:\n{}\n",
                        debug_msg.to_string(),
                        &out,
                        &remaining // ""
                    );
                }
                Ok((remaining, out))
            }
            Err(nom::Err::Error(error) | nom::Err::Failure(error)) => {
                if print {
                    debug!(
                        "Fail branch: {}: {:#?}\nRemaining:\n{}",
                        debug_msg.to_string(),
                        &error,
                        &error.input
                    );
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
