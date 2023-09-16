use std::{cell::RefCell, ops::Range};

use nom::combinator::all_consuming;

use self::utils::debug_fn;

use super::Document;

pub(crate) mod utils;

/// This used in place of `&str` or `&[u8]` in our `nom` parsers.
pub(crate) type LocatedSpan<'a> = nom_locate::LocatedSpan<&'a str, State>;
/// Convenient type alias for `nom::IResult<I, O>` reduced to `IResult<O>`.
pub(crate) type IResult<'a, T> = nom::IResult<LocatedSpan<'a>, T>;

trait ToRange {
    fn to_range(&self) -> Range<usize>;
}

impl<'a> ToRange for LocatedSpan<'a> {
    fn to_range(&self) -> Range<usize> {
        let start = self.location_offset();
        let end = start + self.fragment().len();
        start..end
    }
}

/// Error containing a text span and an error message to display.
#[derive(Debug, Clone)]
pub struct Error(Range<usize>, String);

/// Carried around in the `LocatedSpan::extra` field in
/// between `nom` parsers.
#[derive(Clone, Debug)]
pub struct State(pub RefCell<Vec<Error>>);

impl State {
    /// Pushes an error onto the errors stack from within a `nom`
    /// parser combinator while still allowing parsing to continue.
    pub fn report_error(&self, error: Error) {
        self.0.borrow_mut().push(error);
    }
}

/// A trait with a function that implements parsing to the type
pub trait CSTParse<'c, O> {
    /// Parse `O` from the input
    #[must_use]
    fn parse(input: LocatedSpan<'c>) -> IResult<O>;
}

/// Parses a string into a document struct, also emmitting errors along the way
pub fn parse(source: &str) -> (Document, Vec<Error>) {
    let errors = RefCell::new(Vec::new());
    let input = LocatedSpan::new_extra(source, State(errors));
    let (span, doc) = match all_consuming(debug_fn(Document::parse, "Got Document", true))(input) {
        Ok(it) => it,
        Err(err) => {
            dbg!(err);
            panic!()
        }
    };
    // let res = Document::parse(input);
    // dbg!(res);
    // panic!();
    let a = span.into_fragment_and_extra();
    (doc, a.1 .0.into_inner())
}
