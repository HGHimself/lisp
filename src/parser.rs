use crate::{char_to_operator, Expression, Operator};
use nom::{
    branch::alt,
    character::complete::{char, multispace0, one_of},
    combinator::{all_consuming, map},
    error::{context, ErrorKind, ParseError, VerboseError},
    multi::{fold_many1, many0},
    number::complete::double,
    sequence::{delimited, preceded},
    Err::Error,
    IResult,
};

#[derive(Debug, PartialEq)]
pub enum CustomError<I> {
    MyError,
    Nom(I, ErrorKind),
}

impl<I> ParseError<I> for CustomError<I> {
    fn from_error_kind(input: I, kind: ErrorKind) -> Self {
        CustomError::Nom(input, kind)
    }

    fn append(_: I, _: ErrorKind, other: Self) -> Self {
        other
    }
}

fn parse_number(s: &str) -> IResult<&str, Expression> {
    map(context("number", preceded(multispace0, double)), |n| {
        Expression::Num(n)
    })(s)
}

fn parse_operator(s: &str) -> IResult<&str, Expression> {
    map(
        context("operator", preceded(multispace0, one_of("+-*/"))),
        |c| Expression::Op(char_to_operator(c)),
    )(s)
}

fn parse_arguements(s: &str) -> IResult<&str, Expression> {
    match parse_operator(s) {
        IResult::Ok((remaining, operator)) => map(
            fold_many1(
                parse_expression,
                move || vec![operator.clone()],
                |mut acc: Vec<_>, item| {
                    acc.push(item);
                    acc
                },
            ),
            |v| Expression::Exp(v),
        )(remaining),
        Err(e) => IResult::Err(e),
    }
}

fn parse_expression(s: &str) -> IResult<&str, Expression> {
    alt((
        parse_number,
        delimited(
            preceded(multispace0, char('(')),
            parse_arguements,
            preceded(multispace0, char(')')),
        ),
    ))(s)
}

pub fn parse(s: &str) -> IResult<&str, Expression> {
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
    fn it_parses_all_operators() {
        assert_eq!(parse_operator("+"), Ok(("", Expression::Op(Operator::Add))));
        assert_eq!(
            parse_operator("\t-"),
            Ok(("", Expression::Op(Operator::Sub)))
        );
        assert_eq!(
            parse_operator("  *"),
            Ok(("", Expression::Op(Operator::Mul)))
        );
        assert_eq!(
            parse_operator("\n/"),
            Ok(("", Expression::Op(Operator::Div)))
        );
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
                    Expression::Op(Operator::Mul),
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
                    Expression::Op(Operator::Mul),
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
                    Expression::Op(Operator::Mul),
                    Expression::Num(1_f64),
                    Expression::Num(2_f64),
                    Expression::Exp(vec!(
                        Expression::Op(Operator::Mul),
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
                    Expression::Op(Operator::Mul),
                    Expression::Num(9_f64),
                    Expression::Exp(vec!(
                        Expression::Op(Operator::Mul),
                        Expression::Num(1_f64),
                        Expression::Num(2_f64),
                        Expression::Exp(vec!(
                            Expression::Op(Operator::Mul),
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
