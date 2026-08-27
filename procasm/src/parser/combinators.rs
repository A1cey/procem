use crate::parser::{ParserError, ParserInput, ParserState};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    NoMatch(ParserError),
    IncompleteMatch(ParserError),
}

impl Error {
    #[must_use]
    #[inline]
    pub fn into_no_match(self) -> Self {
        match self {
            Self::NoMatch(_) => self,
            Self::IncompleteMatch(err) => Self::NoMatch(err),
        }
    }

    #[must_use]
    #[inline]
    pub fn into_incomplete_match(self) -> Self {
        match self {
            Self::IncompleteMatch(_) => self,
            Self::NoMatch(err) => Self::IncompleteMatch(err),
        }
    }

    #[must_use]
    #[inline]
    pub fn inner(self) -> ParserError {
        match self {
            Self::IncompleteMatch(err) | Self::NoMatch(err) => err,
        }
    }
}

pub trait Parser<'input> {
    type Output;

    fn parse(self, input: ParserInput<'input>, state: &mut ParserState<'input>) -> Result<Self::Output, Error>;

    #[inline]
    fn and<P2>(self, other: P2) -> And<Self, P2>
    where
        Self: Sized,
    {
        And(self, other)
    }

    #[inline]
    fn or<P2>(self, other: P2) -> Or<Self, P2>
    where
        Self: Sized,
    {
        Or(self, other)
    }

    #[inline]
    fn left(self) -> Left<Self>
    where
        Self: Sized,
    {
        Left(self)
    }

    #[inline]
    fn right(self) -> Right<Self>
    where
        Self: Sized,
    {
        Right(self)
    }

    #[inline]
    fn map<F, T2>(self, f: F) -> Map<Self, F>
    where
        Self: Sized,
        F: FnOnce(Self::Output) -> T2,
    {
        Map(self, f)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Value<T>(pub T);

impl<T> Parser<'_> for Value<T> {
    type Output = T;

    #[inline]
    fn parse(self, _input: ParserInput<'_>, _state: &mut ParserState<'_>) -> Result<Self::Output, Error> {
        Ok(self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct And<P1, P2>(P1, P2);

impl<'input, P1, P2> Parser<'input> for And<P1, P2>
where
    P1: Parser<'input>,
    P2: Parser<'input>,
{
    type Output = (P1::Output, P2::Output);

    #[inline]
    fn parse(self, input: ParserInput<'input>, state: &mut ParserState<'input>) -> Result<Self::Output, Error> {
        let t1 = self.0.parse(input, state)?;
        let t2 = self.1.parse(input, state)?;
        Ok((t1, t2))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Or<P1, P2>(P1, P2);

impl<'input, P1, P2> Parser<'input> for Or<P1, P2>
where
    P1: Parser<'input>,
    P2: Parser<'input, Output = P1::Output>,
{
    type Output = P1::Output;

    #[inline]
    fn parse(self, input: ParserInput<'input>, state: &mut ParserState<'input>) -> Result<Self::Output, Error> {
        let saved_idx = state.idx;

        match self.0.parse(input, state) {
            Ok(item) => Ok(item),
            Err(Error::IncompleteMatch(err)) => Err(Error::IncompleteMatch(err)),
            Err(Error::NoMatch(_)) => {
                state.idx = saved_idx;
                self.1.parse(input, state)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Left<P>(P);

impl<'input, P, T1, T2> Parser<'input> for Left<P>
where
    P: Parser<'input, Output = (T1, T2)>,
{
    type Output = T1;

    #[inline]
    fn parse(self, input: ParserInput<'input>, state: &mut ParserState<'input>) -> Result<Self::Output, Error> {
        self.0.parse(input, state).map(|(t1, _t2)| t1)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Right<P>(P);

impl<'input, P, T1, T2> Parser<'input> for Right<P>
where
    P: Parser<'input, Output = (T1, T2)>,
{
    type Output = T2;

    #[inline]
    fn parse(self, input: ParserInput<'input>, state: &mut ParserState<'input>) -> Result<Self::Output, Error> {
        self.0.parse(input, state).map(|(_t1, t2)| t2)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Map<P, MapFn>(P, MapFn);

impl<'input, P, MapFn, T2> Parser<'input> for Map<P, MapFn>
where
    P: Parser<'input>,
    MapFn: FnOnce(P::Output) -> T2,
{
    type Output = T2;

    #[inline]
    fn parse(self, input: ParserInput<'input>, state: &mut ParserState<'input>) -> Result<Self::Output, Error> {
        self.0.parse(input, state).map(|res| (self.1)(res))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Check<F>(pub F);

impl<'input, F> Parser<'input> for Check<F>
where
    F: FnOnce(&ParserState<'input>) -> Result<(), Error>,
{
    type Output = ();

    #[inline]
    fn parse(self, _input: ParserInput<'input>, state: &mut ParserState<'input>) -> Result<Self::Output, Error> {
        (self.0)(state)
    }
}
