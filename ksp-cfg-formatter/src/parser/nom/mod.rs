use std::{cell::RefCell, ops::Range};

use nom::combinator::all_consuming;

use super::{document::source_file, Document};

pub(crate) mod utils;

/// This used in place of `&str` or `&[u8]` in our `nom` parsers.
pub(crate) type LocatedSpan<'a> = nom_locate::LocatedSpan<&'a str, State<'a>>;
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
pub struct Error<'a> {
    /// The LocatedSpan covered by the error
    pub span: LocatedSpan<'a>,
    /// The error message
    pub message: String,
}

/// Holds the state of the parser, to allow for context aware parsing
#[derive(Clone, Debug)]
pub struct ParserState {
    /// Indicates if the current node is on the top level
    pub top_level: bool,
}

impl Default for ParserState {
    fn default() -> Self {
        Self { top_level: true }
    }
}

/// Carried around in the `LocatedSpan::extra` field in
/// between `nom` parsers.
#[derive(Clone, Debug)]
pub struct State<'a> {
    /// List of accumulated errors while parsing
    pub errors: RefCell<Vec<Error<'a>>>,
    /// The current state of the parser
    pub state: ParserState,
}

impl<'a> Default for State<'a> {
    fn default() -> State<'a> {
        State {
            errors: RefCell::new(Vec::new()),
            state: Default::default(),
        }
    }
}

impl<'a> State<'a> {
    /// Pushes an error onto the errors stack from within a `nom`
    /// parser combinator while still allowing parsing to continue.
    pub fn report_error(&self, error: Error<'a>) {
        self.errors.borrow_mut().push(error);
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
    let input = LocatedSpan::new_extra(source, State::default());
    let (span, doc) = all_consuming(source_file)(input).expect("parsing cannot fail");
    let (_, state) = span.into_fragment_and_extra();
    let errors = state.errors.borrow().clone();
    (doc, errors)
}
