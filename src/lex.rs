use std::collections::HashMap;

use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take_while1},
    character::complete::{char, digit1, multispace0, one_of},
    combinator::{eof, map, map_res, opt, recognize, value},
    sequence::{delimited, preceded, tuple},
    IResult,
};
use once_cell::sync::Lazy;

static UNIT_TOKEN: Lazy<HashMap<&'static str, Token>> = Lazy::new(|| {
    HashMap::from_iter([
        ("true", Token::True),
        ("false", Token::False),
        ("nil", Token::Nil),
        ("and", Token::And),
        ("break", Token::Break),
        ("do", Token::Do),
        ("else", Token::Else),
        ("elseif", Token::Elseif),
        ("end", Token::End),
        ("for", Token::For),
        ("function", Token::Function),
        ("goto", Token::Goto),
        ("if", Token::If),
        ("in", Token::In),
        ("local", Token::Local),
        ("not", Token::Not),
        ("or", Token::Or),
        ("return", Token::Return),
        ("then", Token::Then),
        ("while", Token::While),
        ("repeat", Token::Repeat),
        ("until", Token::Until),
        ("<<", Token::ShiftL),
        (">>", Token::ShiftR),
        ("//", Token::Idiv),
        ("==", Token::Equal),
        ("~=", Token::NotEq),
        ("<=", Token::LesEq),
        (">=", Token::GreEq),
        ("::", Token::DoubColon),
        ("..", Token::Concat),
        ("...", Token::Dots),
        ("+", Token::Add),
        ("-", Token::Sub),
        ("*", Token::Mul),
        ("/", Token::Div),
        ("%", Token::Mod),
        ("^", Token::Pow),
        ("#", Token::Len),
        ("&", Token::BitAnd),
        ("~", Token::BitXor),
        ("|", Token::BitOr),
        ("<", Token::Less),
        (">", Token::Greater),
        ("=", Token::Assign),
        ("(", Token::ParL),
        (")", Token::ParR),
        ("{", Token::CurlyL),
        ("}", Token::CurlyR),
        ("[", Token::SqurL),
        ("]", Token::SqurR),
        (";", Token::SemiColon),
        (":", Token::Colon),
        (",", Token::Comma),
        (".", Token::Dot),
    ])
});

#[derive(Debug)]
pub struct Lexer<'a> {
    source: &'a str,
}

#[rustfmt::skip]
#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    // keywords
    And,    Break,  Do,     Else,   Elseif, End,
    False,  For,    Function, Goto, If,     In,
    Local,  Nil,    Not,    Or,     Repeat, Return,
    Then,   True,   Until,  While,

 // +       -       *       /       %       ^       #
    Add,    Sub,    Mul,    Div,    Mod,    Pow,    Len,
 // &       ~       |       <<      >>      //
    BitAnd, BitXor, BitOr,  ShiftL, ShiftR, Idiv,
 // ==       ~=     <=      >=      <       >        =
    Equal,  NotEq,  LesEq,  GreEq,  Less,   Greater, Assign,
 // (       )       {       }       [       ]       ::
    ParL,   ParR,   CurlyL, CurlyR, SqurL,  SqurR,  DoubColon,
 // ;               :       ,       .       ..      ...
    SemiColon,      Colon,  Comma,  Dot,    Concat, Dots,

    // constant values
    Integer(i64),
    Float(f64),
    String(String),

    // name of variables or table keys
    Name(String),

    // end
    Eof,
}

pub type LexError = nom::Err<nom::error::Error<String>>;

impl<'a> Lexer<'a> {
    pub fn new(s: &'a str) -> Self {
        Self { source: s }
    }

    pub fn next(&mut self) -> Result<Token, LexError> {
        lex(self.source)
            .map(|(input, output)| {
                self.source = input;
                output
            })
            .map_err(|e| e.to_owned())
    }
}

fn lex(s: &str) -> IResult<&str, Token> {
    preceded(
        multispace0,
        alt((
            lex_string,
            lex_float,
            lex_integer,
            lex_word,
            lex_chars,
            value(Token::Eof, eof),
        )),
    )(s)
}

fn lex_integer(s: &str) -> IResult<&str, Token> {
    map_res(recognize(preceded(opt(char('-')), digit1)), |s: &str| {
        s.parse().map(Token::Integer)
    })(s)
}

fn lex_float(s: &str) -> IResult<&str, Token> {
    map_res(
        alt((
            // Case one: .42
            recognize(tuple((
                char('.'),
                digit1,
                opt(tuple((one_of("eE"), opt(one_of("+-")), digit1))),
            ))),
            // Case two: 42e42 and 42.42e42
            recognize(tuple((
                digit1,
                opt(preceded(char('.'), digit1)),
                one_of("eE"),
                opt(one_of("+-")),
                digit1,
            ))),
            // Case three: 42. and 42.42
            recognize(tuple((digit1, char('.'), opt(digit1)))),
        )),
        |s: &str| s.parse().map(Token::Float),
    )(s)
}

fn lex_string(s: &str) -> IResult<&str, Token> {
    map(delimited(char('"'), is_not("\""), char('"')), |s: &str| {
        Token::String(s.to_owned())
    })(s)
}

fn lex_word(s: &str) -> IResult<&str, Token> {
    take_while1(|c: char| c.is_ascii_alphanumeric() || c == '_')(s).map(|(input, output)| {
        (
            input,
            UNIT_TOKEN
                .get(output)
                .cloned()
                .unwrap_or_else(|| Token::Name(String::from(output))),
        )
    })
}

fn lex_chars(s: &str) -> IResult<&str, Token> {
    alt((
        tag("<<"),
        tag(">>"),
        tag("//"),
        tag("=="),
        tag("~="),
        tag("<="),
        tag(">="),
        tag("::"),
        tag(".."),
        tag("..."),
        recognize(one_of("+-*/%^#&~|<>=(){}[];:,.")),
    ))(s)
    .map(|(input, output)| (input, UNIT_TOKEN.get(output).cloned().unwrap()))
}
