use crate::ast::{BinOp, Expr, PREFIX_BP, UnOp};
use crate::lexer::{Tok, Token, lex};
use proto123_coord::{RefAddr, RefSyntax};
use proto123_value::ErrorKind;

/// Longest accepted formula body, in characters.
pub const MAX_LEN: usize = 8_192;
/// Deepest accepted nesting of parentheses, calls, and prefix operators.
pub const MAX_DEPTH: usize = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParseErrorKind {
    /// A token that cannot start or continue an expression here.
    Unexpected,
    UnexpectedEnd,
    UnclosedParen,
    /// A character or token that belongs to no grammar rule.
    UnknownToken,
    TooLong,
    TooDeep,
}

/// A syntax error with the character index where it was detected.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParseError {
    pub pos: usize,
    pub kind: ParseErrorKind,
}

pub(crate) fn parse_body(body: &str) -> Result<Expr, ParseError> {
    let chars: Vec<char> = body.chars().collect();
    if chars.len() > MAX_LEN {
        return Err(ParseError { pos: MAX_LEN, kind: ParseErrorKind::TooLong });
    }
    let tokens = lex(&chars)?;
    let mut p = Parser { tokens, i: 0, depth: 0, end: chars.len() };
    if p.tokens.is_empty() {
        return Err(ParseError { pos: 0, kind: ParseErrorKind::UnexpectedEnd });
    }
    let expr = p.parse_bp(0)?;
    match p.peek_token() {
        None => Ok(expr),
        Some(t) => Err(ParseError { pos: t.pos, kind: ParseErrorKind::Unexpected }),
    }
}

struct Parser {
    tokens: Vec<Token>,
    i: usize,
    depth: usize,
    end: usize,
}

impl Parser {
    fn peek_token(&self) -> Option<&Token> {
        self.tokens.get(self.i)
    }

    fn peek(&self) -> Option<&Tok> {
        self.peek_token().map(|t| &t.tok)
    }

    fn bump(&mut self) -> Option<Token> {
        let t = self.tokens.get(self.i).cloned();
        self.i += 1;
        t
    }

    fn error_here(&self, kind: ParseErrorKind) -> ParseError {
        let pos = self.peek_token().map_or(self.end, |t| t.pos);
        ParseError { pos, kind }
    }

    fn enter(&mut self) -> Result<(), ParseError> {
        self.depth += 1;
        if self.depth > MAX_DEPTH {
            return Err(self.error_here(ParseErrorKind::TooDeep));
        }
        Ok(())
    }

    fn leave(&mut self) {
        self.depth = self.depth.saturating_sub(1);
    }

    /// Pratt loop: parses an expression whose infix operators bind at least `min_bp`.
    fn parse_bp(&mut self, min_bp: u8) -> Result<Expr, ParseError> {
        self.enter()?;
        let result = self.parse_bp_inner(min_bp);
        self.leave();
        result
    }

    fn parse_bp_inner(&mut self, min_bp: u8) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_prefix()?;
        loop {
            let op = match self.peek() {
                Some(Tok::Op('+')) => BinOp::Add,
                Some(Tok::Op('-')) => BinOp::Sub,
                Some(Tok::Op('*')) => BinOp::Mul,
                Some(Tok::Op('/')) => BinOp::Div,
                Some(Tok::Op('^')) => BinOp::Pow,
                _ => break,
            };
            let (lbp, rbp) = op.binding_power();
            if lbp < min_bp {
                break;
            }
            self.bump();
            let rhs = self.parse_bp(rbp)?;
            lhs = Expr::Binary(op, Box::new(lhs), Box::new(rhs));
        }
        Ok(lhs)
    }

    fn parse_prefix(&mut self) -> Result<Expr, ParseError> {
        let op = match self.peek() {
            Some(Tok::Op('-')) => Some(UnOp::Neg),
            Some(Tok::Op('+')) => Some(UnOp::Plus),
            _ => None,
        };
        if let Some(op) = op {
            self.bump();
            let operand = self.parse_bp(PREFIX_BP)?;
            return Ok(Expr::Unary(op, Box::new(operand)));
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        // `@` before a function name is accepted shorthand: `@SUM(...)` == `SUM(...)`.
        if matches!(self.peek(), Some(Tok::At)) {
            let at = self.bump().map_or(self.end, |t| t.pos);
            let is_call = matches!(self.tokens.get(self.i), Some(Token { tok: Tok::Word(_), .. }))
                && matches!(self.tokens.get(self.i + 1), Some(Token { tok: Tok::LParen, .. }));
            if !is_call {
                return Err(ParseError { pos: at, kind: ParseErrorKind::Unexpected });
            }
        }
        let Some(token) = self.bump() else {
            return Err(ParseError { pos: self.end, kind: ParseErrorKind::UnexpectedEnd });
        };
        match token.tok {
            Tok::Number(n) => Ok(Expr::Number(n)),
            Tok::ErrorLit(kind) => Ok(Expr::ErrorLit(kind)),
            Tok::LParen => {
                let inner = self.parse_bp(0)?;
                self.expect_rparen()?;
                Ok(inner)
            }
            Tok::Word(word) => self.parse_word(word),
            _ => Err(ParseError { pos: token.pos, kind: ParseErrorKind::Unexpected }),
        }
    }

    fn expect_rparen(&mut self) -> Result<(), ParseError> {
        match self.peek() {
            Some(Tok::RParen) => {
                self.bump();
                Ok(())
            }
            None => Err(ParseError { pos: self.end, kind: ParseErrorKind::UnclosedParen }),
            Some(_) => Err(self.error_here(ParseErrorKind::Unexpected)),
        }
    }

    fn parse_word(&mut self, word: String) -> Result<Expr, ParseError> {
        if matches!(self.peek(), Some(Tok::LParen)) {
            return self.parse_call(word);
        }
        if word.eq_ignore_ascii_case("TRUE") {
            return Ok(Expr::Bool(true));
        }
        if word.eq_ignore_ascii_case("FALSE") {
            return Ok(Expr::Bool(false));
        }
        match RefAddr::classify(&word) {
            RefSyntax::Valid(first) => {
                if matches!(self.peek(), Some(Tok::Colon)) {
                    self.bump();
                    return self.parse_range_end(first);
                }
                Ok(Expr::Ref(first))
            }
            RefSyntax::OutOfGrid => {
                // Out-of-grid references are `#REF!` values, not syntax errors (R8).
                if matches!(self.peek(), Some(Tok::Colon)) {
                    self.bump();
                    self.parse_range_end_lenient()?;
                }
                Ok(Expr::ErrorLit(ErrorKind::Ref))
            }
            RefSyntax::NotARef => {
                // A bare name (not a function call) is an unknown name: `#NAME?`.
                Ok(Expr::ErrorLit(ErrorKind::Name))
            }
        }
    }

    fn parse_range_end(&mut self, first: RefAddr) -> Result<Expr, ParseError> {
        let Some(token) = self.bump() else {
            return Err(ParseError { pos: self.end, kind: ParseErrorKind::UnexpectedEnd });
        };
        let Tok::Word(word) = token.tok else {
            return Err(ParseError { pos: token.pos, kind: ParseErrorKind::Unexpected });
        };
        match RefAddr::classify(&word) {
            RefSyntax::Valid(second) => Ok(Expr::Range(first, second)),
            RefSyntax::OutOfGrid => Ok(Expr::ErrorLit(ErrorKind::Ref)),
            RefSyntax::NotARef => Err(ParseError { pos: token.pos, kind: ParseErrorKind::Unexpected }),
        }
    }

    fn parse_range_end_lenient(&mut self) -> Result<(), ParseError> {
        let Some(token) = self.bump() else {
            return Err(ParseError { pos: self.end, kind: ParseErrorKind::UnexpectedEnd });
        };
        match token.tok {
            Tok::Word(word) if !matches!(RefAddr::classify(&word), RefSyntax::NotARef) => Ok(()),
            _ => Err(ParseError { pos: token.pos, kind: ParseErrorKind::Unexpected }),
        }
    }

    fn parse_call(&mut self, word: String) -> Result<Expr, ParseError> {
        // Consume '('.
        self.bump();
        let mut args = Vec::new();
        if matches!(self.peek(), Some(Tok::RParen)) {
            self.bump();
            return Ok(Expr::Call { name: word.to_ascii_uppercase(), args });
        }
        loop {
            args.push(self.parse_bp(0)?);
            match self.peek() {
                Some(Tok::Sep) => {
                    self.bump();
                }
                Some(Tok::RParen) => {
                    self.bump();
                    break;
                }
                None => return Err(ParseError { pos: self.end, kind: ParseErrorKind::UnclosedParen }),
                Some(_) => return Err(self.error_here(ParseErrorKind::Unexpected)),
            }
        }
        Ok(Expr::Call { name: word.to_ascii_uppercase(), args })
    }
}
