use crate::{char_to_symbol, Expression, Symbol};
use nom::{
    branch::alt,
    character::complete::{char, multispace0, one_of},
    combinator::{all_consuming, map},
    error::{context, ErrorKind, ParseError},
    multi::{fold_many1, many0},
    number::complete::double,
    sequence::{delimited, preceded},
    Err::Error,
    IResult,
};

#[derive(Debug, PartialEq)]
pub enum SyntaxError<I> {
    InvalidArguments,
    InvalidSymbol,
    Nom(I, ErrorKind),
}

impl<I> ParseError<I> for SyntaxError<I> {
    fn from_error_kind(input: I, kind: ErrorKind) -> Self {
        SyntaxError::Nom(input, kind)
    }

    fn append(_: I, _: ErrorKind, other: Self) -> Self {
        other
    }
}

fn parse_number(s: &str) -> IResult<&str, Expression, SyntaxError<&str>> {
    map(preceded(multispace0, double), |n| Expression::Num(n))(s)
}

fn parse_symbol(s: &str) -> IResult<&str, Expression, SyntaxError<&str>> {
    map(preceded(multispace0, one_of("+-*/")), |c| {
        Expression::Sym(char_to_symbol(c))
    })(s)
}

fn parse_arguements(s: &str) -> IResult<&str, Expression, SyntaxError<&str>> {
    match parse_symbol(s) {
        Ok((rest, symbol)) => {
            let r = fold_many1(
                parse_expression,
                move || vec![symbol.clone()],
                |mut acc: Vec<_>, item| {
                    acc.push(item);
                    acc
                },
            )(rest);
            match r {
                Ok((rest, v)) => Ok((rest, Expression::Exp(v))),
                Err(_) => Err(Error(SyntaxError::InvalidArguments)),
            }
        }
        Err(e) => Err(Error(SyntaxError::InvalidSymbol)),
    }
}

fn parse_expression(s: &str) -> IResult<&str, Expression, SyntaxError<&str>> {
    alt((
        parse_number,
        delimited(
            preceded(multispace0, char('(')),
            parse_arguements,
            preceded(multispace0, char(')')),
        ),
    ))(s)
}

pub fn parse(s: &str) -> IResult<&str, Expression, SyntaxError<&str>> {
    all_consuming(delimited(multispace0, parse_arguements, multispace0))(s)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_parses_numbers() {
        assert_eq!(parse_number("1"), Ok(("", Expression::Num(1.0_f64))));
        assert_eq!(
            parse_number("1.000001-1"),
            Ok(("-1", Expression::Num(1.000001_f64)))
        );
        assert_eq!(parse_number("123E-02"), Ok(("", Expression::Num(1.23_f64))));
        assert_eq!(
            parse_number("-12302"),
            Ok(("", Expression::Num(-12302_f64)))
        );
        assert_eq!(parse_number("  \t1"), Ok(("", Expression::Num(1_f64))));
    }

    #[test]
    fn it_parses_all_symbols() {
        assert_eq!(parse_symbol("+"), Ok(("", Expression::Sym(Symbol::Add))));
        assert_eq!(parse_symbol("\t-"), Ok(("", Expression::Sym(Symbol::Sub))));
        assert_eq!(parse_symbol("  *"), Ok(("", Expression::Sym(Symbol::Mul))));
        assert_eq!(parse_symbol("\n/"), Ok(("", Expression::Sym(Symbol::Div))));
    }

    #[test]
    fn it_parses_arguements() {
        assert_eq!(
            parse_arguements(
                "* 1
             2 3"
            ),
            Ok((
                "",
                Expression::Exp(vec!(
                    Expression::Sym(Symbol::Mul),
                    Expression::Num(1_f64),
                    Expression::Num(2_f64),
                    Expression::Num(3_f64),
                ))
            ))
        );
    }

    #[test]
    fn it_parses_an_expression() {
        assert_eq!(
            parse_expression(
                "(* 1
             2 3)"
            ),
            Ok((
                "",
                Expression::Exp(vec!(
                    Expression::Sym(Symbol::Mul),
                    Expression::Num(1_f64),
                    Expression::Num(2_f64),
                    Expression::Num(3_f64),
                ))
            ))
        );

        assert_eq!(
            parse_expression(
                "(* 1
             2 (* 1
          2 3))"
            ),
            Ok((
                "",
                Expression::Exp(vec!(
                    Expression::Sym(Symbol::Mul),
                    Expression::Num(1_f64),
                    Expression::Num(2_f64),
                    Expression::Exp(vec!(
                        Expression::Sym(Symbol::Mul),
                        Expression::Num(1_f64),
                        Expression::Num(2_f64),
                        Expression::Num(3_f64),
                    )),
                ))
            ))
        );

        assert_eq!(
            parse_expression(
                "9 (* 1
             2 (* 1
          2 3))"
            ),
            Ok((
                " (* 1\n             2 (* 1\n          2 3))",
                Expression::Num(9_f64)
            ))
        );
    }

    #[test]
    fn it_parses_expressions() {
        assert_eq!(
            parse(
                "* 9 (* 1
             2 (* 1
          2 3))"
            ),
            Ok((
                "",
                Expression::Exp(vec!(
                    Expression::Sym(Symbol::Mul),
                    Expression::Num(9_f64),
                    Expression::Exp(vec!(
                        Expression::Sym(Symbol::Mul),
                        Expression::Num(1_f64),
                        Expression::Num(2_f64),
                        Expression::Exp(vec!(
                            Expression::Sym(Symbol::Mul),
                            Expression::Num(1_f64),
                            Expression::Num(2_f64),
                            Expression::Num(3_f64),
                        )),
                    )),
                ))
            ))
        );
    }
}
