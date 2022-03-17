use crate::ast::{GroupItem, Value};

use nom::{
    branch::alt,
    bytes::complete::{is_a, is_not, tag, take_until, take_while},
    character::complete::{alpha1, char, line_ending, multispace0, one_of},
    combinator::{all_consuming, cut, map, map_res, opt, peek, recognize},
    error::{context, ParseError},
    multi::{fold_many0, separated_list},
    number::complete::double,
    sequence::{delimited, preceded, terminated, tuple},
    IResult,
};

fn underscore_tag<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&str, &str, E> {
    context(
        "underscore_tag",
        recognize(preceded(
            alpha1,
            take_while(|c: char| c.is_alphanumeric() || c.eq_ignore_ascii_case(&'_')),
        )),
    )(input)
}

fn quoted_floats<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&str, Vec<f64>, E> {
    context(
        "quoted floats",
        preceded(
            char('\"'),
            terminated(
                separated_list(
                    preceded(multispace0, char(',')),
                    preceded(multispace0, double),
                ),
                char('\"'),
            ),
        ),
    )(input)
}

fn expression<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&str, &str, E> {
    context("expression", move |input| {
        recognize(separated_list(
            // operator
            preceded(multispace0, is_a("+-*/")),
            // operand
            preceded(
                multispace0,
                preceded(
                    opt(is_a("-")),
                    alt((
                        // sub expression
                        preceded(char('('), cut(terminated(expression, char(')')))),
                        // identifier
                        underscore_tag,
                        // constant
                        recognize(double),
                    )),
                ),
            ),
        ))(input)
    })(input)
}

fn quoted_string<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&str, &str, E> {
    context(
        "quoted string",
        preceded(char('\"'), cut(terminated(is_not("\""), char('\"')))),
    )(input)
}

fn boolean<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&str, bool, E> {
    map_res(alpha1, |s: &str| s.parse::<bool>())(input)
}

fn simple_attr_value<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&str, Value, E> {
    context(
        "simple attr value",
        preceded(
            multispace0,
            alt((
                map(quoted_floats, Value::FloatGroup),
                map(quoted_string, |s| Value::String(s.to_string())),
                map(terminated(double, peek(one_of(",; \t)"))), Value::Float),
                map(boolean, Value::Bool),
                map(map(expression, String::from), Value::Expression),
            )),
        ),
    )(input)
}

fn simple_attribute<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&str, GroupItem, E> {
    context(
        "simple attr",
        map(
            tuple((
                preceded(multispace0, underscore_tag),
                preceded(multispace0, char(':')),
                cut(preceded(multispace0, simple_attr_value)),
                preceded(multispace0, char(';')),
            )),
            |(name, _, value, _)| GroupItem::SimpleAttr(name.to_string(), value),
        ),
    )(input)
}

fn complex_attribute_values<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&str, Vec<Value>, E> {
    context(
        "complex values",
        delimited(
            preceded(multispace0, tag("(")),
            delimited(
                opt(tuple((multispace0, tag("\\"), line_ending))),
                separated_list(
                    alt((
                        map(
                            tuple((multispace0, tag(","), multispace0, tag("\\"), line_ending)),
                            |_| Some(1),
                        ),
                        map(tuple((multispace0, tag(","))), |_| Some(1)),
                        map(
                            tuple((multispace0, tag("\\"), line_ending, multispace0, tag(","))),
                            |_| Some(1),
                        ),
                    )),
                    preceded(multispace0, simple_attr_value),
                ),
                opt(tuple((multispace0, tag("\\"), line_ending))),
            ),
            preceded(multispace0, tag(")")),
        ),
    )(input)
}

fn complex_attribute<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&str, GroupItem, E> {
    context(
        "complex attr",
        map(
            tuple((
                preceded(multispace0, underscore_tag),
                preceded(multispace0, complex_attribute_values),
                preceded(multispace0, char(';')),
            )),
            |(name, value, _)| GroupItem::ComplexAttr(name.to_string(), value),
        ),
    )(input)
}

fn comment<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&str, &str, E> {
    context(
        "comment",
        recognize(delimited(tag("/*"), take_until("*/"), tag("*/"))),
    )(input)
}

fn parse_group_body<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&str, Vec<GroupItem>, E> {
    context(
        "group body",
        fold_many0(
            context(
                "folding items",
                alt((
                    map(
                        map(preceded(multispace0, comment), String::from),
                        GroupItem::Comment,
                    ),
                    preceded(multispace0, parse_group),
                    preceded(multispace0, simple_attribute),
                    preceded(multispace0, complex_attribute),
                )),
            ),
            Vec::new(),
            |mut acc: Vec<_>, item| {
                acc.push(item);
                acc
            },
        ),
    )(input)
}
fn parse_group<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&str, GroupItem, E> {
    context(
        "parsing group",
        map(
            tuple((
                preceded(multispace0, underscore_tag),
                preceded(
                    preceded(multispace0, char('(')),
                    terminated(
                        map(
                            separated_list(
                                preceded(multispace0, char(',')),
                                preceded(
                                    multispace0,
                                    alt((
                                        map(quoted_string, |s| format!("\"{}\"", s)),
                                        map(underscore_tag, |s| format!("{}", s)),
                                    )),
                                ),
                            ),
                            |vals: Vec<String>| vals.join(", "),
                        ),
                        preceded(multispace0, char(')')),
                    ),
                ),
                preceded(
                    preceded(multispace0, char('{')),
                    cut(terminated(
                        parse_group_body,
                        preceded(multispace0, char('}')),
                    )),
                ),
            )),
            |(gtype, name, body)| GroupItem::Group(gtype.to_string(), name, body),
        ),
    )(input)
}

pub fn parse_libs<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&str, Vec<GroupItem>, E> {
    context(
        "parse_libs",
        all_consuming(terminated(
            fold_many0(
                alt((
                    context(
                        "outer comment",
                        map(
                            map(delimited(multispace0, comment, multispace0), String::from),
                            GroupItem::Comment,
                        ),
                    ),
                    preceded(multispace0, context("parse_lib", parse_group)),
                )),
                Vec::new(),
                |mut acc: Vec<_>, item| {
                    match &item {
                        GroupItem::Group(_, _, _) => acc.push(item),
                        GroupItem::Comment(_) => {}
                        GroupItem::SimpleAttr(_, _) => {}
                        GroupItem::ComplexAttr(_, _) => {}
                    }
                    acc
                },
            ),
            multispace0,
        )),
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom::{
        error::{convert_error, ErrorKind, VerboseError},
        Err,
    };

    #[test]
    fn test_complex_attr_values() {
        assert_eq!(
            complex_attribute_values::<VerboseError<&str>>(
                r#"( \
    "0, 0.18, 0.33", \
    "-0.555, -0.45, -0.225" \
    )"#
            ),
            Ok((
                "",
                vec![
                    Value::FloatGroup(vec![0.0, 0.18, 0.33]),
                    Value::FloatGroup(vec![-0.555, -0.45, -0.225]),
                ]
            ))
        );
        assert_eq!(
            complex_attribute_values::<VerboseError<&str>>(
                r#"( \
                 "a string(b)" \
                 )"#
            ),
            Ok(("", vec![Value::String("a string(b)".to_string())]))
        );
        assert_eq!(
            complex_attribute_values::<VerboseError<&str>>("(123,-456)"),
            Ok(("", vec![Value::Float(123.0), Value::Float(-456.0),]))
        );
    }

    #[test]
    fn test_complex_attr() {
        assert_eq!(
            complex_attribute::<(&str, ErrorKind)>("capacitive_load_unit (1,pf);"),
            Ok((
                "",
                GroupItem::ComplexAttr(
                    "capacitive_load_unit".to_string(),
                    vec![Value::Float(1.0), Value::Expression("pf".to_string()),],
                )
            ))
        );
    }

    #[test]
    fn test_complex_attr_multi_line() {
        assert_eq!(
            complex_attribute::<(&str, ErrorKind)>(
                r#"values ( \
    "0, 0.18, 0.33", \
    "-0.555, -0.45, -0.225");"#
            ),
            Ok((
                "",
                GroupItem::ComplexAttr(
                    "values".to_string(),
                    vec![
                        Value::FloatGroup(vec![0.0, 0.18, 0.33]),
                        Value::FloatGroup(vec![-0.555, -0.45, -0.225]),
                    ],
                )
            ))
        );
    }

    #[test]
    fn test_comment() {
        assert_eq!(
            comment::<(&str, ErrorKind)>("/*** abc **/def"),
            Ok(("def", "/*** abc **/"))
        );
        assert_eq!(
            comment::<(&str, ErrorKind)>(
                "/* multi
line
**
**/
**/rest"
            ),
            Ok((
                "
**/rest",
                "/* multi
line
**
**/"
            ))
        );
    }

    #[test]
    fn test_underscore_tag() {
        assert_eq!(
            underscore_tag::<(&str, ErrorKind)>("a_b__c"),
            Ok(("", "a_b__c"))
        );
        assert_eq!(
            underscore_tag::<(&str, ErrorKind)>("abc other"),
            Ok((" other", "abc"))
        );
        assert_eq!(
            underscore_tag::<(&str, ErrorKind)>("nand2"),
            Ok(("", "nand2"))
        );
        assert_eq!(
            underscore_tag::<(&str, ErrorKind)>("_"),
            Err(Err::Error(("_", ErrorKind::Alpha)))
        );
        assert_eq!(
            underscore_tag::<(&str, ErrorKind)>(" a_b"),
            Err(Err::Error((" a_b", ErrorKind::Alpha)))
        );
        assert_eq!(
            underscore_tag::<(&str, ErrorKind)>(",,"),
            Err(Err::Error((",,", ErrorKind::Alpha)))
        );
    }

    #[test]
    fn test_simple_attribute_malformed() {
        assert_eq!(
            simple_attribute::<(&str, ErrorKind)>("attr_name : a b ; "),
            Err(Err::Error(("b ; ", ErrorKind::Char))),
        );
    }
    #[test]
    fn test_simple_attribute_bool() {
        assert_eq!(
            simple_attribute::<(&str, ErrorKind)>("attr_name : true ; "),
            Ok((
                " ",
                GroupItem::SimpleAttr(String::from("attr_name"), Value::Bool(true),)
            ))
        );
        assert_eq!(
            simple_attribute::<(&str, ErrorKind)>("attr_name : false ; "),
            Ok((
                " ",
                GroupItem::SimpleAttr(String::from("attr_name"), Value::Bool(false),)
            ))
        );
    }
    #[test]
    fn test_simple_attribute_float() {
        assert_eq!(
            simple_attribute::<(&str, ErrorKind)>("attr_name : 345.123 ; "),
            Ok((
                " ",
                GroupItem::SimpleAttr(String::from("attr_name"), Value::Float(345.123),)
            ))
        );
        assert_eq!(
            simple_attribute::<(&str, ErrorKind)>("attr_name : -345.123 ; "),
            Ok((
                " ",
                GroupItem::SimpleAttr(String::from("attr_name"), Value::Float(-345.123),)
            ))
        );
    }
    #[test]
    fn test_simple_attribute_int() {
        assert_eq!(
            simple_attribute::<(&str, ErrorKind)>("attr_name : 345 ; "),
            Ok((
                " ",
                GroupItem::SimpleAttr(String::from("attr_name"), Value::Float(345.0),)
            ))
        );
        assert_eq!(
            simple_attribute::<(&str, ErrorKind)>("attr_name : -345 ; "),
            Ok((
                " ",
                GroupItem::SimpleAttr(String::from("attr_name"), Value::Float(-345.0),)
            ))
        );
    }

    #[test]
    fn test_expression() {
        let expressions = vec![
            "A",
            "-A",
            "--A",
            "A + B",
            "A - B",
            "A * B",
            "A / B",
            "(A + B)",
            "((A + B))",
            "(A / B) + C",
            "(A / B) - (C * D)",
            /****** with constants ******/
            "A + 0.123",
            "A - .123",
            "A * 0",
            "A / 1",
            "(A + 0.5)",
            "((1.23 + B))",
            "(4.5 / B) + C",
            "(7.8 / B) - (C * D)",
        ];
        for expr in expressions {
            assert_eq!(expression::<(&str, ErrorKind)>(expr), Ok(("", expr)));
        }
    }

    #[test]
    fn test_simple_attribute_expression() {
        assert_eq!(
            simple_attribute::<(&str, ErrorKind)>("attr_name : nand2; "),
            Ok((
                " ",
                GroupItem::SimpleAttr(
                    String::from("attr_name"),
                    Value::Expression(String::from("nand2")),
                )
            ))
        );
        assert_eq!(
            simple_attribute::<(&str, ErrorKind)>("attr_name : table_lookup; "),
            Ok((
                " ",
                GroupItem::SimpleAttr(
                    String::from("attr_name"),
                    Value::Expression(String::from("table_lookup")),
                )
            ))
        );
        let data = "attr_name : A +B; ";
        match simple_attribute::<VerboseError<&str>>(data) {
            Err(Err::Error(err)) | Err(Err::Failure(err)) => {
                println!("Error: {}", convert_error(data, err));
                assert_eq!(true, false);
            }
            _ => {}
        }
        assert_eq!(
            simple_attribute::<(&str, ErrorKind)>("attr_name : A + 1.2; "),
            Ok((
                " ",
                GroupItem::SimpleAttr(
                    String::from("attr_name"),
                    Value::Expression(String::from("A + 1.2")),
                )
            ))
        );
    }

    #[test]
    fn test_simple_attribute_string() {
        assert_eq!(
            simple_attribute::<(&str, ErrorKind)>("attr_name : \"table_lookup\"; "),
            Ok((
                " ",
                GroupItem::SimpleAttr(
                    String::from("attr_name"),
                    Value::String(String::from("table_lookup"))
                )
            ))
        );
    }

    #[test]
    fn test_parse_group() {
        let data = "library ( foo ) {
            abc ( 1, 2, 3 );
        }";
        match parse_group::<VerboseError<&str>>(data) {
            Err(Err::Error(err)) | Err(Err::Failure(err)) => {
                println!("Error: {}", convert_error(data, err));
                assert_eq!(true, false);
            }
            _ => {}
        };
        assert_eq!(
            parse_group::<(&str, ErrorKind)>(data),
            Ok((
                "",
                GroupItem::Group(
                    "library".to_string(),
                    "foo".to_string(),
                    vec![GroupItem::ComplexAttr(
                        "abc".to_string(),
                        vec![Value::Float(1.0), Value::Float(2.0), Value::Float(3.0),],
                    ),],
                ),
            ))
        );
    }

    #[test]
    fn test_nested_group() {
        assert_eq!(
            parse_group::<(&str, ErrorKind)>(
                r#"
            outer( outer ) {
                inner ( inner) {
                    abc ( 1, 2, 3 );
                }
                inner(inner2 ) {
                    abc ( 1, 2, 3 );
                }
            }"#
            ),
            Ok((
                "",
                GroupItem::Group(
                    "outer".to_string(),
                    "outer".to_string(),
                    vec![
                        GroupItem::Group(
                            "inner".to_string(),
                            "inner".to_string(),
                            vec![GroupItem::ComplexAttr(
                                "abc".to_string(),
                                vec![Value::Float(1.0), Value::Float(2.0), Value::Float(3.0),],
                            ),],
                        ),
                        GroupItem::Group(
                            "inner".to_string(),
                            "inner2".to_string(),
                            vec![GroupItem::ComplexAttr(
                                "abc".to_string(),
                                vec![Value::Float(1.0), Value::Float(2.0), Value::Float(3.0),],
                            ),],
                        ),
                    ]
                )
            ))
        );
    }

    #[test]
    fn test_lib_simple() {
        assert_eq!(
            parse_libs::<(&str, ErrorKind)>(
                r#"
/*
 delay model :       typ
 check model :       typ
 power model :       typ
 capacitance model : typ
 other model :       typ
*/
library(foo) {

  delay_model : table_lookup;
  /* unit attributes */
  time_unit : "1ns";
  capacitive_load_unit (1, pf );
  function: "A & B";

  slew_upper_threshold_pct_rise : 80;
  nom_temperature : 25.0;
}
"#
            ),
            Ok((
                "",
                vec![GroupItem::Group(
                    "library".to_string(),
                    "foo".to_string(),
                    vec![
                        GroupItem::SimpleAttr(
                            "delay_model".to_string(),
                            Value::Expression("table_lookup".to_string())
                        ),
                        GroupItem::Comment("/* unit attributes */".to_string()),
                        GroupItem::SimpleAttr(
                            "time_unit".to_string(),
                            Value::String("1ns".to_string())
                        ),
                        GroupItem::ComplexAttr(
                            "capacitive_load_unit".to_string(),
                            vec![Value::Float(1.0), Value::Expression("pf".to_string()),],
                        ),
                        GroupItem::SimpleAttr(
                            "function".to_string(),
                            Value::String("A & B".to_string()),
                        ),
                        GroupItem::SimpleAttr(
                            "slew_upper_threshold_pct_rise".to_string(),
                            Value::Float(80.0)
                        ),
                        GroupItem::SimpleAttr("nom_temperature".to_string(), Value::Float(25.0)),
                    ],
                ),]
            ))
        );
    }
}
