
use super::object::*;
use std::cell::Cell;

#[allow(dead_code)]


#[derive(PartialEq, Debug)]
enum Token {
    Lpar, Rpar,
    Dot, Quote,
    Integer(i32),
    Float(f64),
    Symbol(String),
    String(String),
    End
}


#[derive(Debug)]
pub struct ParseErr (String);


const SYMBOLS_ALLOWED: &str = "+-.*/<=>!?:$%_&~^";


fn parse_number(source : &[char]) -> Result<(usize, Token), ParseErr> {
    let mut ptr = 1;
    let mut real = false;
    while ptr < source.len() {
        let c = source[ptr];
        if c == '.' {
            if real {
                return Err(ParseErr("Wrong floating point formatting".to_string()));
            }
            real = true;
        } else if !c.is_digit(10) {
            break;
        }
        ptr += 1;
    }
    let s : String = source[..ptr].into_iter().collect();
    if real {
        return Ok((ptr - 1, Token::Float(s.parse().unwrap())));
    } else {
        return Ok((ptr - 1, Token::Integer(s.parse().unwrap())));
    }
}


fn parse_string(source : &[char]) -> Result<(usize, Token), ParseErr> {
    let mut ptr = 1;
    loop {
        if ptr < source.len() {
            if source[ptr] == '"' {
                return Ok((ptr + 1, Token::String(source[..ptr].into_iter().collect())));
            }
            ptr += 1;
        } else {
            return Err(ParseErr("String literal didn't close".to_string()));
        }
    }
}


fn parse_symbol(source : &[char]) -> (usize, Token) {
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


fn tokenize(source : &str) -> Result<Vec<Token>, ParseErr> {
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
            }
            else if c == '(' {
                result.push(Token::Lpar);
            }
            else if c == ')' {
                result.push(Token::Rpar);
            }
            else if c == '\'' {
                result.push(Token::Quote);
            }
            else if c == '.' {
                result.push(Token::Dot);
            }
            else if c.is_digit(10) {
                match parse_number(&chars[ptr..]) {
                    Ok((p, token)) => {
                        ptr += p;
                        result.push(token);
                    },
                    Err(e) => return Err(e)
                };
            }
            else if c == '"' {
                match parse_string(&chars[ptr+1..]) {
                    Ok((p, token)) => {
                        ptr += p;
                        result.push(token);
                    },
                    Err(e) => return Err(e)
                };
            }
            else if c.is_alphabetic() || SYMBOLS_ALLOWED.contains(c) {
                let r = parse_symbol(&chars[ptr..]);
                ptr += r.0;
                result.push(r.1);
            }
            ptr += 1;
        } else {
            result.push(Token::End);
            break;
        }
    }
    Ok(result)
}

/**
  The language's grammar:

  program  ::=  object* End
  object   ::=  (list | 'object | atom
  list     ::=  object list | ) | .object)
  atom     ::=  number | symbol | string
 */


/**
 * The main parsing function.
 */
pub fn parse_expression(source : &str) -> Result<Vec<Object>, ParseErr> {
    let tokens = tokenize(source);
    if let Err(e) = tokens {
        return Err(e);
    }
    let tokens = &mut tokens.unwrap().into_iter();
    let mut program = Vec::<Object>::new();
    loop {
        if let Some(t) = tokens.next() {
            if t == Token::End {
                break;
            }
            let current = Cell::new(Object::Nil);
            if let Some (e) = parse_object(&current, t, tokens) {
                return Err(e);
            }
            program.push(current.take());
        }
    }
    Ok(program)
}

/**
 * object  ::=  (list
 * object  ::=  'object
 * object  ::=  number | symbol | string
 */
fn parse_object(current : &Cell<Object>, first: Token, rest : &mut Iterator<Item=Token>)
            -> Option<ParseErr> {
    match first {
        Token::Symbol(s) => current.set(Object::Symbol(s)),
        Token::String(s) => current.set(Object::String(s)),
        Token::Float(value) => current.set(Object::Number(Number::Float(value))),
        Token::Integer(value) => current.set(Object::Number(Number::Integer(value))),
        Token::Quote => {
            if let Some(t) = rest.next() {
                match parse_object(current, t, rest) {
                    None => {
                        current.set(Object::make_pair(
                                Object::Symbol("quote".to_string()),
                                Object::make_pair(current.take(), Object::Nil)));
                    }
                    Some(e) => return Some(e)
                }
            } else {
                return Some(ParseErr("Unexpected end of input".to_string()));
            }
        },
        Token::Lpar => {
            return match rest.next() {
                Some(t) => parse_list(current, t, rest),
                None => Some(ParseErr("Unexpected end of input after opening parenthesis".to_string()))
            }
        },
        _ => return Some(ParseErr(format!("Unexpected token: {:?}", first)))
    };
    return None;
}

/**
 * list  ::=  )
 * list  ::=  . object)
 * list  ::=  object list
 */
fn parse_list(current : &Cell<Object>, first: Token, rest : &mut Iterator<Item=Token>)
            -> Option<ParseErr> {
    match first {
        Token::Rpar => None,
        Token::Dot => {
            if let Some(t) = rest.next() {
                if let Some(e) = parse_object(current, t, rest) {
                    return Some(e);
                }
            } else {
                return Some(ParseErr("Unexpected end of input after a dot".to_string()));
            }
            match rest.next() {
                Some(Token::Rpar) => None,
                Some(t) => Some(ParseErr(format!("Closing parenthesis expected ({:?} found)", t))),
                None => Some(ParseErr("Closing parenthesis expected (found end of input)".to_string()))
            }
        },
        _ => {
            if let Some(e) = parse_object(current, first, rest) {
                return Some(e);
            }
            let head = current.take();
            if let Some(t) = rest.next() {
                if let Some(e) = parse_list(current, t, rest) {
                    return Some(e);
                }
                current.set(Object::make_pair(head, current.take()));
            } else {
                return Some(ParseErr("Unexpected end of input".to_string()));
            }
            None
        }
    }
}

pub fn debug_expression(input : &str) {
    dbg!(tokenize(input));
    dbg!(parse_expression(input));
}


#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn lexer_test() {
        assert_eq!(tokenize(" \t \n ; qqq ").unwrap(),
                    vec![Token::End]);

        assert_eq!(tokenize("(.')").unwrap(),
                    vec![Token::Lpar, Token::Dot, Token::Quote, Token::Rpar, Token::End]);

        assert_eq!(tokenize("\"str\" symbol").unwrap(),
                    vec![Token::String("str".to_string()),
                        Token::Symbol("symbol".to_string()), Token::End]);

        assert_eq!(tokenize("42 3.14").unwrap(),
                    vec![Token::Integer(42), Token::Float(3.14), Token::End]);

        assert_eq!(tokenize("\"❤\"").unwrap()[0], Token::String("❤".to_string()));

        assert!(tokenize("\"   ").is_err());
        assert!(tokenize("1.23.456").is_err());
    }

    #[test]
    fn parser_test() {
        assert_eq!(parse_expression("2").unwrap()[0],
                    Object::Number(Number::Integer(2)));
                    
        assert_eq!(parse_expression("(1 . a)").unwrap()[0],
                    Object::make_pair(Object::Number(Number::Integer(1)),
                                      Object::Symbol("a".to_string())));

        assert!(parse_expression("(").is_err());
        assert!(parse_expression("(1 .").is_err());
        assert!(parse_expression("(1 . 2 .").is_err());
    }

}
