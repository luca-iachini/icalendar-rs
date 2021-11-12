use std::fmt::{self, Write};

use crate::properties::fold_line;

use super::{
    parameters::{parameters, Parameter},
    utils::property_key,
};
use nom::{
    bytes::complete::{tag, take_until},
    character::complete::{line_ending, multispace0},
    combinator::{cut, map, opt},
    error::{context, ContextError, ParseError},
    sequence::{preceded, separated_pair, tuple},
    IResult,
};

#[cfg(test)]
use nom::error::ErrorKind;

/// Zero-copy version of [`crate::properties::Property`]
#[derive(PartialEq, Debug, Clone)]
pub struct Property<'a> {
    pub key: &'a str,
    pub val: &'a str,
    pub params: Vec<Parameter<'a>>,
}

impl Property<'_> {
    pub fn fmt_write<W: Write>(&self, out: &mut W) -> Result<(), fmt::Error> {
        // A nice starting capacity for the majority of content lines
        let mut line = String::with_capacity(150);

        write!(line, "{}", self.key)?;
        for &Parameter { ref key, ref val } in &self.params {
            if let Some(val) = val {
                write!(line, ";{}={}", key, val)?;
            } else {
                write!(line, ";{}", key)?;
            }
        }
        write!(line, ":{}", self.val)?;
        write_crlf!(out, "{}", fold_line(&line))?;
        Ok(())
    }
}

impl From<Property<'_>> for crate::Property {
    fn from(parsed: Property<'_>) -> Self {
        Self::new(parsed.key, parsed.val)
    }
}

#[test]
fn test_property() {
    assert_parser!(
        property,
        "KEY:VALUE\n",
        Property {
            key: "KEY",
            val: "VALUE",
            params: vec![]
        }
    );

    assert_parser!(
        property,
        "KEY1;foo=bar:VALUE\n",
        Property {
            key: "KEY1",
            val: "VALUE",
            params: vec![Parameter {
                key: "foo",
                val: Some("bar")
            }]
        }
    );

    assert_parser!(
        property,
        "KEY2;foo=bar:VALUE space separated\n",
        Property {
            key: "KEY2",
            val: "VALUE space separated",
            params: vec![Parameter {
                key: "foo",
                val: Some("bar")
            }]
        }
    );

    assert_parser!(
        property,
        "KEY2;foo=bar:important:VALUE\n",
        Property {
            key: "KEY2",
            val: "important:VALUE",
            params: vec![Parameter {
                key: "foo",
                val: Some("bar")
            }]
        }
    );

    // TODO: newlines followed by spaces must be ignored
    assert_parser!(
        property,
        "KEY3;foo=bar:VALUE\\n newline separated\n",
        Property {
            key: "KEY3",
            val: "VALUE\\n newline separated",
            params: vec![Parameter {
                key: "foo",
                val: Some("bar")
            }]
        }
    );
}

#[test]
#[rustfmt::skip]
fn parse_properties_from_rfc() {
    // TODO: newlines followed by spaces must be ignored
    assert_parser!(
        property,
        "home.tel;type=fax,voice,msg:+49 3581 123456\n",
        Property {
            key: "home.tel",
            val: "+49 3581 123456",
            params: vec![Parameter {
                key: "type",
                val: Some("fax,voice,msg"),
            }]
        }
    );
    // TODO: newlines followed by spaces must be ignored
    assert_parser!(
        property,
        "email;internet:mb@goerlitz.de\n",
        Property {
            key: "email",
            val: "mb@goerlitz.de",
            params: vec![Parameter {
                key: "internet"  ,
                val: None,
            }]
        }
    );
}

#[test]
#[rustfmt::skip]
fn parse_property_with_breaks() {

    let sample_0 = "DESCRIPTION:Hey, I'm gonna have a party\\n BYOB: Bring your own beer.\\n Hendrik\\n\n";

    let expectation = Property {
        key: "DESCRIPTION",
        val: "Hey, I'm gonna have a party\\n BYOB: Bring your own beer.\\n Hendrik\\n",
        params: vec![]
    };

    assert_parser!(property, sample_0, expectation);
}

pub fn property<'a, E: ParseError<&'a str> + ContextError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, Property, E> {
    context(
        "property",
        cut(map(
            tuple((
                separated_pair(
                    tuple((
                        // preceded(multispace0, alpha_or_dash), // key
                        cut(context(
                            "property can't be END or BEGIN",
                            preceded(multispace0, property_key),
                        )), // key
                        parameters, // params
                    )),
                    context("property sparator", tag(":")), // separator
                    context("property value", take_until("\n")), // val TODO: replace this with something simpler!
                ),
                opt(line_ending),
            )),
            |(((key, params), val), _)| Property { key, val, params },
        )),
    )(input)
}
