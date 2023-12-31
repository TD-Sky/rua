use std::str;

use nom::{
    branch::alt,
    bytes::{
        complete::{is_not, take, take_while},
        streaming::take_while_m_n,
    },
    character::{complete::char, streaming::multispace1},
    combinator::{map, map_res, value, verify},
    multi::fold_many0,
    sequence::{delimited, preceded},
    IResult, Parser,
};
use tinyvec::TinyVec;

use super::Token;
use crate::str::LossyStr;

pub fn lex_string(input: &str) -> IResult<&str, Token> {
    let build_string = fold_many0(
        fragment,
        TinyVec::<[u8; LossyStr::INLINE_CAP]>::new,
        |mut string, fragment| {
            match fragment {
                StringFragment::Literal(s) => string.extend_from_slice(s.as_bytes()),
                StringFragment::EscapedChar(c) => string.push(c),
                StringFragment::EscapedWS => {}
            }
            string
        },
    );

    map(delimited(char('"'), build_string, char('"')), Token::String)(input)
}

/// A string fragment contains a fragment of a string being parsed:
///
/// - a non-empty Literal (a series of non-escaped characters)
/// - a single parsed Escaped Character
/// - a block of Escaped Whitespace
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StringFragment<'a> {
    Literal(&'a str),
    EscapedChar(u8),
    EscapedWS,
}

fn fragment(input: &str) -> IResult<&str, StringFragment<'_>> {
    alt((
        map(literal, StringFragment::Literal),
        map(escaped_char, StringFragment::EscapedChar),
        value(StringFragment::EscapedWS, escaped_whitespace),
    ))(input)
}

/// Parse a non-empty block of text that doesn't include \ or "
fn literal(input: &str) -> IResult<&str, &str> {
    // 若输入满足`F`，则用`G`验证，通过则返回输入，否则返回验证错误；
    // 若输入不满足`F`，则返回`F`的错误。
    verify(is_not(r#""\"#), |s: &str| !s.is_empty())(input)
}

/// Parse an escaped character
// \a   bell
// \b   back space
// \f   form feed
// \n   newline
// \r   carriage return
// \t   horizontal tab
// \v   vertical tab
// \\   backslash
// \"   double quote
// \'   single quote
// \nnn byte (0 ~ 255)
fn escaped_char(input: &str) -> IResult<&str, u8> {
    preceded(
        char('\\'),
        alt((
            value(b'\n', char('n')),
            value(b'\r', char('r')),
            value(b'\t', char('t')),
            value(b'\x07', char('a')),
            value(b'\x08', char('b')),
            value(b'\x0B', char('v')),
            value(b'\x0C', char('f')),
            dec_byte,
            hex_byte,
            value(b'\\', char('\\')),
            value(b'\'', char('\'')),
            value(b'"', char('"')),
        )),
    )(input)
}

/// Parse a backslash, followed by any amount of whitespace.
/// This is used to discard any escaped whitespace.
fn escaped_whitespace(input: &str) -> IResult<&str, &str> {
    preceded(char('\\'), multispace1)(input)
}

fn dec_byte(input: &str) -> IResult<&str, u8> {
    let dec = take_while_m_n(1, 3, |n: char| n.is_ascii_digit());
    map_res(dec, |s: &str| s.parse())(input)
}

fn hex_byte(input: &str) -> IResult<&str, u8> {
    let hex = take(2usize).and_then(take_while(|n: char| n.is_ascii_hexdigit()));
    let preceded_hex = preceded(char('x'), hex);
    map_res(preceded_hex, |s: &str| u8::from_str_radix(s, 16))(input)
}
