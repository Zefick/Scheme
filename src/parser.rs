
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
struct ParseErr (String);


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
                break;
            }
            ptr += 1;
        } else {
            return Err(ParseErr("String literal didn't close".to_string()));
        }
    }
    return Ok((ptr + 1, Token::String(source[..ptr].into_iter().collect())));
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

}
