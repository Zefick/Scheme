use super::object::*;
use crate::errors::ParseErr;
use std::fmt::Debug;

#[derive(PartialEq, Debug)]
enum Token {
    Lpar,
    Rpar,
    Dot,
    Quote,
    Integer(i32),
    Float(f64),
    Symbol(String),
    String(String),
}

impl ToString for Token {
    fn to_string(&self) -> String {
        match self {
            Token::Lpar => "(".to_string(),
            Token::Rpar => ")".to_string(),
            Token::Dot => ".".to_string(),
            Token::Quote => "'".to_string(),
            Token::Integer(x) => x.to_string(),
            Token::Float(x) => x.to_string(),
            Token::Symbol(x) => x.into(),
            Token::String(x) => x.into(),
        }
    }
}

const SYMBOLS_ALLOWED: &str = "+-.*/<=>!?:$%_&~^#";

fn parse_number(source: &[char]) -> Result<(usize, Token), ParseErr> {
    let mut ptr = 1;
    let mut real = false;
    while ptr < source.len() {
        let c = source[ptr];
        if c == '.' {
            if real {
                return Err(ParseErr::WrongFP);
            }
            real = true;
        } else if !c.is_digit(10) {
            break;
        }
        ptr += 1;
    }
    let s: String = source[..ptr].into_iter().collect();
    if real {
        return Ok((ptr - 1, Token::Float(s.parse().unwrap())));
    } else {
        return Ok((ptr - 1, Token::Integer(s.parse().unwrap())));
    }
}

fn parse_string(source: &[char]) -> Result<(usize, Token), ParseErr> {
    let mut ptr = 0;
    loop {
        if ptr < source.len() {
            if source[ptr] == '"' {
                return Ok((ptr + 1, Token::String(source[..ptr].into_iter().collect())));
            }
            ptr += 1;
        } else {
            return Err(ParseErr::UnclosedString);
        }
    }
}

fn parse_symbol(source: &[char]) -> (usize, Token) {
    let mut ptr = 1;
    while ptr < source.len() {
        let c = source[ptr];
        if c.is_digit(10) || c.is_alphabetic() || SYMBOLS_ALLOWED.contains(c) {
            ptr += 1;
        } else {
            break;
        }
    }
    return (ptr - 1, Token::Symbol(source[..ptr].into_iter().collect()));
}

fn tokenize(source: &str) -> Result<Vec<Token>, ParseErr> {
    let chars = &source.chars().collect::<Vec<char>>()[..];
    let mut result = Vec::<Token>::new();
    let mut ptr = 0;
    loop {
        if ptr < chars.len() {
            let c = chars[ptr];
            if c == ';' {
                while ptr < chars.len() && chars[ptr] != '\n' {
                    ptr += 1;
                }
            } else if c == '(' {
                result.push(Token::Lpar);
            } else if c == ')' {
                result.push(Token::Rpar);
            } else if c == '\'' {
                result.push(Token::Quote);
            } else if c == '.' {
                result.push(Token::Dot);
            } else if c.is_digit(10) {
                let (p, token) = parse_number(&chars[ptr..])?;
                ptr += p;
                result.push(token);
            } else if c == '"' {
                let (p, token) = parse_string(&chars[ptr + 1..])?;
                ptr += p;
                result.push(token);
            } else if c.is_alphabetic() || SYMBOLS_ALLOWED.contains(c) {
                let r = parse_symbol(&chars[ptr..]);
                ptr += r.0;
                result.push(r.1);
            }
            ptr += 1;
        } else {
            break;
        }
    }
    Ok(result)
}

/*
 The language's grammar:

 program  ::=  object* End
 object   ::=  (list | 'object | atom
 list     ::=  object list | ) | .object)
 atom     ::=  number | symbol | string
*/

/**
 * The main parsing function.
 */
pub fn parse_expression(source: &str) -> Result<Vec<Object>, ParseErr> {
    let tokens = &mut tokenize(source)?.into_iter();
    let mut program = Vec::<Object>::new();
    while let Some(t) = tokens.next() {
        program.push(parse_object(t, tokens)?);
    }
    Ok(program)
}

/*
 * object  ::=  (list
 * object  ::=  'object
 * object  ::=  number | symbol | string
 */
fn parse_object(first: Token, rest: &mut dyn Iterator<Item = Token>) -> Result<Object, ParseErr> {
    match first {
        Token::Symbol(s) => Ok(Object::Symbol(s)),
        Token::String(s) => Ok(Object::String(s)),
        Token::Float(value) => Ok(Object::Number(Number::Float(value))),
        Token::Integer(value) => Ok(Object::Number(Number::Integer(value))),
        Token::Quote => (rest.next())
            .map(|token| {
                let current = parse_object(token, rest)?;
                Ok(Object::make_pair(
                    Object::Symbol("quote".to_string()),
                    Object::make_pair(current, Object::Nil),
                ))
            })
            .unwrap_or_else(|| Err(ParseErr::Unexpected_EOF)),
        Token::Lpar => (rest.next())
            .map(|token| parse_list(token, rest))
            .unwrap_or_else(|| Err(ParseErr::Unexpected_EOF_AfterPars)),
        _ => Err(ParseErr::UnexpectedToken(format!("{:?}", first))),
    }
}

/*
 * list  ::=  )
 * list  ::=  . object)
 * list  ::=  object list
 */
fn parse_list(first: Token, rest: &mut dyn Iterator<Item = Token>) -> Result<Object, ParseErr> {
    match first {
        Token::Rpar => Ok(Object::Nil),
        Token::Dot => (rest.next())
            .map(|token| {
                let current = parse_object(token, rest)?;
                match rest.next() {
                    Some(Token::Rpar) => Ok(current),
                    Some(t) => Err(ParseErr::ClosingParExpected(t.to_string())),
                    None => Err(ParseErr::ClosingParExpected_EOF),
                }
            })
            .unwrap_or_else(|| Err(ParseErr::Unexpected_EOF_AfterDot)),
        _ => {
            let head = parse_object(first, rest)?;
            rest.next()
                .map(|token| Ok(Object::make_pair(head, parse_list(token, rest)?)))
                .unwrap_or_else(|| Err(ParseErr::Unexpected_EOF))
        }
    }
}

#[cfg(test)]
#[rustfmt::skip]
mod tests {
    use super::*;

    fn expect_err(source: &str, expected: ParseErr) {
        let result = parse_expression(source);
        match result {
            Ok(_) => {
                panic!("Error \"{}\" expected for \"{}\"", expected, source)
            }
            Err(thrown) => {
                if thrown != expected {
                    panic!("Error \"{}\" expected for \"{}\", \"{}\" thrown", expected, source, thrown)
                }
            }
        }
    }

    #[test]
    fn lexer_test() {
        assert!(tokenize(" \t \n ; qqq ").unwrap().is_empty());

        assert_eq!(tokenize("(.')").unwrap(),
                   vec![Token::Lpar, Token::Dot, Token::Quote, Token::Rpar]);

        assert_eq!(tokenize("\"str\" symbol").unwrap(),
                   vec![Token::String("str".to_string()),
                        Token::Symbol("symbol".to_string())]);

        assert_eq!(tokenize("42 3.14").unwrap(),
                   vec![Token::Integer(42), Token::Float(3.14)]);

        assert_eq!(tokenize("\"\"").unwrap(), vec![Token::String("".to_string())]);
        assert_eq!(tokenize("\"❤\"").unwrap(), vec![Token::String("❤".to_string())]);

        expect_err("\"   ", ParseErr::UnclosedString);
        expect_err("1.23.456", ParseErr::WrongFP);
    }

    #[test]
    fn parser_test() {
        assert_eq!(parse_expression("2").unwrap(), vec![Object::make_int(2)]);

        assert_eq!(parse_expression("()").unwrap(), vec![Object::Nil]);

        assert_eq!(parse_expression("(1)").unwrap(),
                   vec![Object::make_pair(Object::make_int(1), Object::Nil)]);

        assert_eq!(parse_expression("(1 . a)").unwrap(),
                   vec![Object::make_pair(Object::make_int(1),
                                          Object::Symbol("a".to_string()))]);

        assert_eq!(parse_expression("1 (2 3) ()").unwrap(),
                   vec![Object::make_int(1),
                        Object::make_pair(Object::make_int(2),
                                          Object::make_pair(Object::make_int(3), Object::Nil)),
                        Object::Nil]);

        expect_err("(", ParseErr::Unexpected_EOF_AfterPars);
        expect_err("(1 .", ParseErr::Unexpected_EOF_AfterDot);
        expect_err("(1 . 2 .", ParseErr::ClosingParExpected(".".to_string()));
    }
}
