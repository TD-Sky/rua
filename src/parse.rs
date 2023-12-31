use smol_str::SmolStr;

use self::error::{bail, expect_next};
use crate::{ByteCode, ByteCodeStack, Lexer, Token, Value};

#[derive(Debug)]
pub struct ParseProto<'a> {
    pub constants: Vec<Value>,
    pub bytecodes: Vec<ByteCode>,
    pub lexer: Lexer<'a>,
    pub locals: Vec<SmolStr>,
}

impl<'a> ParseProto<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            constants: Vec::default(),
            bytecodes: Vec::default(),
            lexer: Lexer::new(source),
            locals: Vec::default(),
        }
    }

    pub fn parse(mut self) -> anyhow::Result<Self> {
        loop {
            let code = match self.lexer.next()? {
                Token::Local => {
                    expect_next!(self.lexer, Token::Name(var), "<variable>");
                    expect_next!(self.lexer, Token::Assign, "`=`");
                    let code = self.load_exp(self.locals.len() as u8)?;
                    self.locals.push(var);
                    code
                }
                Token::Name(name) => match self.lexer.next()? {
                    Token::Assign => self.assign(name),
                    t => self.call_function(t, name),
                }?,
                Token::Eof => break,
                Token::Comment => continue,
                t => bail!(t),
            };
            self.bytecodes.push(code);
        }

        tracing::debug!("constants: {:#?}", self.constants);
        tracing::debug!("bytecode stack: [\n{}]", ByteCodeStack(&self.bytecodes));

        Ok(self)
    }

    fn add_const(&mut self, value: Value) -> usize {
        self.constants
            .iter()
            .position(|v| v == &value)
            .unwrap_or_else(|| {
                self.constants.push(value);
                self.constants.len() - 1
            })
    }

    fn load_const(&mut self, dst: u8, constant: Value) -> ByteCode {
        ByteCode::LoadConst(dst, self.add_const(constant) as u8)
    }

    fn load_exp(&mut self, dst: u8) -> Result<ByteCode, ParseError> {
        let code = match self.lexer.next()? {
            Token::Nil => ByteCode::LoadNil(dst),
            Token::True => ByteCode::LoadBool(dst, true),
            Token::False => ByteCode::LoadBool(dst, false),
            Token::Integer(i) => {
                if let Ok(i) = i16::try_from(i) {
                    ByteCode::LoadInt(dst, i)
                } else {
                    self.load_const(dst, Value::Integer(i))
                }
            }
            Token::Float(f) => self.load_const(dst, Value::Float(f)),
            Token::String(s) => self.load_const(dst, Value::String(s.into())),
            Token::Name(name) => self.load_var(dst, name),
            t => bail!(t, "<expression>"),
        };

        Ok(code)
    }

    fn load_var(&mut self, dst: u8, name: SmolStr) -> ByteCode {
        // 优先查找后定义的变量，即作用域遮蔽
        if let Some(src) = self.local_var(&name) {
            ByteCode::Move(dst, src as u8)
        } else {
            ByteCode::GetGlobal(dst, self.add_const(Value::Identifier(name)) as u8)
        }
    }

    // <local>  = <const>   把常量加载到栈上指定位置，对应字节码 Load*
    // <local>  = <local>   复制栈上值，对应字节码 Move
    // <local>  = <global>  把栈上值赋值给全局变量，对应字节码 GetGlobal
    // <global> = <const>   把常量赋值给全局变量，需要首先把常量加到常量表中，然后通过字节码 SetGlobalConst 完成赋值
    // <global> = <local>   把局部变量赋值给全局变量，对应字节码 SetGlobal
    // <global> = <global>  把全局变量赋值给全局变量，对应字节码 SetGlobalGlobal
    fn assign(&mut self, var: SmolStr) -> Result<ByteCode, ParseError> {
        if let Some(src) = self.local_var(&var) {
            // 正在赋值给局部变量
            self.load_exp(src as u8)
        } else {
            // 正在赋值给全局变量
            let gi = self.add_const(Value::Identifier(var)) as u8;

            let code = match self.lexer.next()? {
                Token::Nil => ByteCode::SetGlobalConst(gi, self.add_const(Value::Nil) as u8),
                Token::True => {
                    ByteCode::SetGlobalConst(gi, self.add_const(Value::Boolean(true)) as u8)
                }
                Token::False => {
                    ByteCode::SetGlobalConst(gi, self.add_const(Value::Boolean(false)) as u8)
                }
                Token::Integer(i) => {
                    ByteCode::SetGlobalConst(gi, self.add_const(Value::Integer(i)) as u8)
                }
                Token::Float(f) => {
                    ByteCode::SetGlobalConst(gi, self.add_const(Value::Float(f)) as u8)
                }
                Token::String(s) => {
                    ByteCode::SetGlobalConst(gi, self.add_const(Value::String(s.into())) as u8)
                }
                Token::Name(var) => {
                    if let Some(src) = self.local_var(&var) {
                        ByteCode::SetGlobalLocal(gi, src as u8)
                    } else {
                        ByteCode::SetGlobalGlobal(gi, self.add_const(Value::Identifier(var)) as u8)
                    }
                }
                t => bail!(t, "<expression>"),
            };

            Ok(code)
        }
    }

    fn local_var(&self, name: &str) -> Option<usize> {
        self.locals.iter().rposition(|var| var == name)
    }

    fn call_function(&mut self, token: Token, name: SmolStr) -> Result<ByteCode, ParseError> {
        let ifunc = self.locals.len() as u8;
        let iarg = ifunc + 1;

        let code = self.load_var(ifunc, name);
        self.bytecodes.push(code);

        match token {
            Token::ParL => {
                let code = self.load_exp(iarg)?;
                self.bytecodes.push(code);
                expect_next!(self.lexer, Token::ParR, "`)`");
            }
            Token::String(s) => {
                let code = self.load_const(iarg, Value::String(s.into()));
                self.bytecodes.push(code);
            }
            t => bail!(t, "`(<expression>)` or string"),
        }

        Ok(ByteCode::Call(ifunc, 1))
    }
}

pub use self::error::{ParseError, UnexpectedTokenError};
mod error {
    use crate::{LexError, Token};

    #[derive(Debug, thiserror::Error)]
    #[error("parse failed: {0}")]
    pub enum ParseError {
        Lex(#[from] LexError),
        Token(#[from] UnexpectedTokenError),
    }

    #[derive(Debug, thiserror::Error)]
    pub struct UnexpectedTokenError {
        pub actual: Token,
        pub expected: &'static str,
    }

    impl std::fmt::Display for UnexpectedTokenError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            if self.expected.is_empty() {
                write!(f, "unexpected token {:?}", self.actual)
            } else {
                write!(
                    f,
                    "expected token {} but got {:?}",
                    self.expected, self.actual
                )
            }
        }
    }

    impl UnexpectedTokenError {
        pub(super) fn new(actual: Token, expected: &'static str) -> Self {
            Self { actual, expected }
        }
    }

    macro_rules! bail {
        ($t:expr) => {
            return Err(UnexpectedTokenError::new($t, "").into())
        };
        ($t:expr, $expected:literal) => {
            return Err(UnexpectedTokenError::new($t, $expected).into())
        };
    }
    pub(super) use bail;

    macro_rules! expect_next {
        ($lexer:expr, $t:pat, $expected:literal) => {
            let next_token = $lexer.next()?;
            let $t = next_token else {
                return Err(UnexpectedTokenError::new(next_token, $expected).into());
            };
        };
    }
    pub(super) use expect_next;
}
