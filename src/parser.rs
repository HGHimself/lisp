use crate::{char_to_symbol, string_to_symbol, Expression};
use nom::{
    branch::alt,
    character::complete::{alphanumeric1, char, multispace0, one_of},
    combinator::{all_consuming, map},
    error::{ErrorKind, ParseError},
    multi::many0,
    number::complete::double,
    sequence::{delimited, preceded},
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
    preceded(
        multispace0,
        alt((
            map(one_of("+-*/"), |c| Expression::Sym(char_to_symbol(c))),
            map(alphanumeric1, |s| Expression::Sym(string_to_symbol(s))),
        )),
    )(s)
}

fn parse_sexpression(s: &str) -> IResult<&str, Expression, SyntaxError<&str>> {
    delimited(
        preceded(multispace0, char('(')),
        map(many0(parse_expression), |e| Expression::Sexp(e)),
        preceded(multispace0, char(')')),
    )(s)
}

fn parse_qexpression(s: &str) -> IResult<&str, Expression, SyntaxError<&str>> {
    delimited(
        preceded(multispace0, char('{')),
        map(many0(parse_expression), |e| Expression::Qexp(e)),
        preceded(multispace0, char('}')),
    )(s)
}

fn parse_expression(s: &str) -> IResult<&str, Expression, SyntaxError<&str>> {
    alt((
        parse_number,
        parse_symbol,
        parse_sexpression,
        parse_qexpression,
    ))(s)
}

pub fn parse(s: &str) -> IResult<&str, Expression, SyntaxError<&str>> {
    all_consuming(delimited(
        multispace0,
        map(many0(parse_expression), |e| Expression::Sexp(e)),
        multispace0,
    ))(s)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Symbol;

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
    fn it_parses_sexpr() {
        assert_eq!(
            parse_sexpression(
                "(* 1
             2 3)"
            ),
            Ok((
                "",
                Expression::Sexp(vec!(
                    Expression::Sym(Symbol::Mul),
                    Expression::Num(1_f64),
                    Expression::Num(2_f64),
                    Expression::Num(3_f64),
                ))
            ))
        );
    }

    #[test]
    fn it_parses_qexpr() {
        assert_eq!(
            parse_qexpression(
                "{* 1
             2 3}"
            ),
            Ok((
                "",
                Expression::Qexp(vec!(
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
                Expression::Sexp(vec!(
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
                Expression::Sexp(vec!(
                    Expression::Sym(Symbol::Mul),
                    Expression::Num(1_f64),
                    Expression::Num(2_f64),
                    Expression::Sexp(vec!(
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
                Expression::Sexp(vec!(
                    Expression::Sym(Symbol::Mul),
                    Expression::Num(9_f64),
                    Expression::Sexp(vec!(
                        Expression::Sym(Symbol::Mul),
                        Expression::Num(1_f64),
                        Expression::Num(2_f64),
                        Expression::Sexp(vec!(
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
