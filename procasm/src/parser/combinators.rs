use crate::parser::{ParserError, ParserInput, ParserState};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    NoMatch(ParserError),
    IncompleteMatch(ParserError),
}

impl Error {
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

    #[inline]
    fn commit(self) -> Commit<Self>
    where
        Self: Sized,
    {
        Commit(self)
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
        let start_state = state.clone();

        match self.0.parse(input, state) {
            Ok(item) => Ok(item),
            // Return IncompleteMatch err immediately
            Err(Error::IncompleteMatch(err)) => Err(Error::IncompleteMatch(err)),
            Err(err_p1) => {
                let end_idx_p1 = state.idx;
                *state = start_state;

                match self.1.parse(input, state) {
                    Ok(item) => Ok(item),
                    // Return IncompleteMatch err immediately
                    Err(Error::IncompleteMatch(err)) => Err(Error::IncompleteMatch(err)),
                    // Return parser 1 err if it parsed equal or more than parser 2 else return
                    // parser 2 err
                    Err(err_p2) => {
                        if end_idx_p1 >= state.idx {
                            Err(err_p1)
                        } else {
                            Err(err_p2)
                        }
                    }
                }
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

#[derive(Debug, Clone)]
pub struct Commit<P>(P);

impl<'input, P> Parser<'input> for Commit<P>
where
    P: Parser<'input>,
{
    type Output = P::Output;

    fn parse(self, input: ParserInput<'input>, state: &mut ParserState<'input>) -> Result<Self::Output, Error> {
        self.0.parse(input, state).map_err(|err| match err {
            Error::IncompleteMatch(_) => err,
            Error::NoMatch(err) => Error::IncompleteMatch(err),
        })
    }
}

#[cfg(test)]
mod test {
    use std::cell::Cell;

    use super::*;
    thread_local! { static FIRST_PARSER_RAN: Cell<bool> = const {Cell::new(false)}; }

    /// Returns 0 or 1 depending on the order of the parsers.
    /// First parser that runs will return 0, second will return 1.
    fn get_parser_idx() -> usize {
        if !FIRST_PARSER_RAN.get() {
            FIRST_PARSER_RAN.set(true);
            false
        } else {
            true
        }
        .into()
    }

    struct NoMatchAfterOneStep;
    impl<'input> Parser<'input> for NoMatchAfterOneStep {
        type Output = usize;
        fn parse(self, _input: ParserInput<'input>, state: &mut ParserState<'input>) -> Result<Self::Output, Error> {
            state.idx += 1;

            // parser reports if they're first or second
            Err(Error::NoMatch(ParserError::TokenNotFound { token_idx: get_parser_idx() }))
        }
    }

    struct NoMatchAfterTwoSteps;
    impl<'input> Parser<'input> for NoMatchAfterTwoSteps {
        type Output = usize;
        fn parse(self, _input: ParserInput<'input>, state: &mut ParserState<'input>) -> Result<Self::Output, Error> {
            state.idx += 2;

            // parser report if their first or second through the end flag
            Err(Error::NoMatch(ParserError::TokenNotFound { token_idx: get_parser_idx() }))
        }
    }

    struct IncompleteMatch;
    impl<'input> Parser<'input> for IncompleteMatch {
        type Output = usize;
        fn parse(self, _input: ParserInput<'input>, state: &mut ParserState<'input>) -> Result<Self::Output, Error> {
            state.idx += 1;

            // parser report if their first or second through the end flag
            Err(Error::IncompleteMatch(ParserError::TokenNotFound { token_idx: get_parser_idx() }))
        }
    }

    struct Match;
    impl<'input> Parser<'input> for Match {
        type Output = usize;
        fn parse(self, _input: ParserInput<'input>, _state: &mut ParserState<'input>) -> Result<Self::Output, Error> {
            // parser report if their first or second through the end flag
            Ok(get_parser_idx())
        }
    }

    mod or {
        use super::*;
        #[test]
        fn no_match_err_parser_1_parses_more() {
            let res = NoMatchAfterTwoSteps
                .or(NoMatchAfterOneStep)
                .parse(ParserInput { raw: b"", tokens: &[] }, &mut ParserState::default())
                .unwrap_err();
            assert_eq!(res, Error::NoMatch(ParserError::TokenNotFound { token_idx: 0 }));
        }

        #[test]
        fn no_match_err_parser_2_parses_more() {
            let res = NoMatchAfterOneStep
                .or(NoMatchAfterTwoSteps)
                .parse(ParserInput { raw: b"", tokens: &[] }, &mut ParserState::default())
                .unwrap_err();
            assert_eq!(res, Error::NoMatch(ParserError::TokenNotFound { token_idx: 1 }));
        }

        #[test]
        fn no_match_err_parse_eq_parser_1_reports() {
            let res = NoMatchAfterOneStep
                .or(NoMatchAfterOneStep)
                .parse(ParserInput { raw: b"", tokens: &[] }, &mut ParserState::default())
                .unwrap_err();
            assert_eq!(res, Error::NoMatch(ParserError::TokenNotFound { token_idx: 0 }));
        }

        #[test]
        fn incomplete_match_always_reports() {
            let res = IncompleteMatch
                .or(NoMatchAfterOneStep)
                .parse(ParserInput { raw: b"", tokens: &[] }, &mut ParserState::default())
                .unwrap_err();
            assert_eq!(res, Error::IncompleteMatch(ParserError::TokenNotFound { token_idx: 0 }));

            FIRST_PARSER_RAN.set(false);
            let res = NoMatchAfterTwoSteps
                .or(IncompleteMatch)
                .parse(ParserInput { raw: b"", tokens: &[] }, &mut ParserState::default())
                .unwrap_err();
            assert_eq!(res, Error::IncompleteMatch(ParserError::TokenNotFound { token_idx: 1 }));

            FIRST_PARSER_RAN.set(false);
            let res =
                IncompleteMatch.or(Match).parse(ParserInput { raw: b"", tokens: &[] }, &mut ParserState::default()).unwrap_err();
            assert_eq!(res, Error::IncompleteMatch(ParserError::TokenNotFound { token_idx: 0 }));
        }

        #[test]
        fn match_over_no_match() {
            assert_eq!(
                NoMatchAfterOneStep.or(Match).parse(ParserInput { raw: b"", tokens: &[] }, &mut ParserState::default()).unwrap(),
                1
            );

            FIRST_PARSER_RAN.set(false);
            assert_eq!(
                Match.or(NoMatchAfterOneStep).parse(ParserInput { raw: b"", tokens: &[] }, &mut ParserState::default()).unwrap(),
                0
            );
        }

        #[test]
        fn first_match() {
            assert_eq!(Match.or(Match).parse(ParserInput { raw: b"", tokens: &[] }, &mut ParserState::default()).unwrap(), 0);
        }

        #[test]
        fn match_over_following_incomplete_match() {
            assert_eq!(
                Match.or(IncompleteMatch).parse(ParserInput { raw: b"", tokens: &[] }, &mut ParserState::default()).unwrap(),
                0
            );
        }
    }
}
