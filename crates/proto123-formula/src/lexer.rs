use crate::parser::{ParseError, ParseErrorKind};
use proto123_value::ErrorKind;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Tok {
    Number(f64),
    /// An identifier or reference-shaped word (`SUM`, `A1`, `$B$2`, `TRUE`).
    Word(String),
    ErrorLit(ErrorKind),
    /// `+ - * / ^`
    Op(char),
    LParen,
    RParen,
    /// `,` or `;`
    Sep,
    Colon,
    At,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Token {
    pub tok: Tok,
    /// Character index in the source.
    pub pos: usize,
}

const ERROR_LITERALS: [(&str, ErrorKind); 5] = [
    ("#REF!", ErrorKind::Ref),
    ("#DIV/0!", ErrorKind::Div0),
    ("#NAME?", ErrorKind::Name),
    ("#VALUE!", ErrorKind::Value),
    ("#CIRC!", ErrorKind::Circ),
];

fn is_word_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '$' || c == '_'
}

fn is_word_part(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '$' | '_' | '.')
}

pub(crate) fn lex(src: &[char]) -> Result<Vec<Token>, ParseError> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < src.len() {
        let c = src.get(i).copied().unwrap_or('\0');
        let pos = i;
        if c.is_whitespace() {
            i += 1;
            continue;
        }
        let tok = match c {
            '+' | '-' | '*' | '/' | '^' => {
                i += 1;
                Tok::Op(c)
            }
            '(' => {
                i += 1;
                Tok::LParen
            }
            ')' => {
                i += 1;
                Tok::RParen
            }
            ',' | ';' => {
                i += 1;
                Tok::Sep
            }
            ':' => {
                i += 1;
                Tok::Colon
            }
            '@' => {
                i += 1;
                Tok::At
            }
            '#' => {
                let rest: String = src.iter().skip(i).collect();
                let hit = ERROR_LITERALS.iter().find(|(text, _)| rest.starts_with(text));
                match hit {
                    Some((text, kind)) => {
                        i += text.chars().count();
                        Tok::ErrorLit(*kind)
                    }
                    None => return Err(ParseError { pos, kind: ParseErrorKind::UnknownToken }),
                }
            }
            c if c.is_ascii_digit() || c == '.' => {
                let start = i;
                while src.get(i).is_some_and(char::is_ascii_digit) {
                    i += 1;
                }
                if src.get(i) == Some(&'.') {
                    i += 1;
                    while src.get(i).is_some_and(char::is_ascii_digit) {
                        i += 1;
                    }
                }
                if matches!(src.get(i), Some('e' | 'E')) {
                    let mut j = i + 1;
                    if matches!(src.get(j), Some('+' | '-')) {
                        j += 1;
                    }
                    if src.get(j).is_some_and(char::is_ascii_digit) {
                        while src.get(j).is_some_and(char::is_ascii_digit) {
                            j += 1;
                        }
                        i = j;
                    }
                }
                let text: String = src.get(start..i).unwrap_or(&[]).iter().collect();
                match text.parse::<f64>() {
                    Ok(n) => Tok::Number(n),
                    Err(_) => return Err(ParseError { pos, kind: ParseErrorKind::UnknownToken }),
                }
            }
            c if is_word_start(c) => {
                let start = i;
                while src.get(i).is_some_and(|&ch| is_word_part(ch)) {
                    i += 1;
                }
                Tok::Word(src.get(start..i).unwrap_or(&[]).iter().collect())
            }
            _ => return Err(ParseError { pos, kind: ParseErrorKind::UnknownToken }),
        };
        out.push(Token { tok, pos });
    }
    Ok(out)
}
