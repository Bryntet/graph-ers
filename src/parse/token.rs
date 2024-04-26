use std::ascii::Char;
use std::collections::VecDeque;
use std::iter::Peekable;
use std::str::Chars;
use std::collections::HashMap;
use itertools::Itertools;
use prse::{Parse, try_parse};
use regex::Regex;
use log::warn;
use crate::parse::math_functions::ParseError;

pub(crate) trait Operation {
    fn do_operation(&self) -> f64;
    fn required_args(&self) -> usize {
        2
    }

    fn operation_type(&self) -> Token;


}

struct Add(f64,f64);

impl Operation for Add {
    fn do_operation(&self) -> f64 {
        self.0+self.1
    }
    fn operation_type(&self) -> Token {
        Token::Add
    }
}
struct Subtract(f64,f64);
impl Operation for Subtract {
    fn do_operation(&self) -> f64 {
        self.0-self.1
    }

    fn operation_type(&self) -> Token {
        Token::Subtract
    }
}

struct Multiply(f64,f64);
impl Operation for Multiply {
    fn do_operation(&self) -> f64 {
        self.0*self.1
    }

    fn operation_type(&self) -> Token {
        Token::Multiply
    }
}

struct Divide(f64,f64);

impl Operation for Divide {
    fn do_operation(&self) -> f64 {
        self.0/self.1
    }

    fn operation_type(&self) -> Token {
        Token::Divide
    }
}

struct Pow(f64, f64);

impl Operation for Pow {
    fn do_operation(&self) -> f64 {
        self.0.powf(self.1)
    }
    fn operation_type(&self) -> Token {
        Token::Pow
    }
}

struct TestFunction(f64,f64);
impl Operation for TestFunction {
    fn do_operation(&self) -> f64 {
        self.0*self.1/2.
    }

    fn operation_type(&self) -> Token {
        Token::TestFunction{a: self.0,b: self.1}
    }
}


#[derive(Parse, Debug, PartialEq, Clone)]
pub(crate) enum Token {
    #[prse = "+"]
    Add,
    #[prse = "-"]
    Subtract,
    #[prse = "*"]
    Multiply,
    #[prse = "/"]
    Divide,
    #[prse = "^"]
    Pow,
    #[prse = "test({a},{b})"]
    TestFunction{
        a: f64,
        b: f64
    },
}
struct Function(FunctionType);
impl Function {
    fn test(&self) {
        println!("Test")
    }

}
enum FunctionType {
    Test
}


impl Token {
    fn new(input: &str) -> Option<Self> {
        let input = input.replace(' ', "");
        try_parse!(&input,"{}").ok()
    }

    fn to_operation(&self, num1:f64, num2: f64) -> Box<dyn Operation> {
        let (n1,n2) = (num1,num2);
        match self {
            Self::Add => Box::new(Add(n1,n2)),
            Self::Subtract => Box::new(Subtract(n1,n2)),
            Self::Multiply => Box::new(Multiply(n1,n2)),
            Self::Divide => Box::new(Divide(n1,n2)),
            Self::Pow => Box::new(Pow(n1,n2)),
            Self::TestFunction{a,b} => Box::new(TestFunction(*a,*b))
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenQueue(Vec<QueueItem>);

#[derive(Debug, Clone, PartialEq)]
enum QueueItem {
    Variable(String),
    Number(f64),
    Token(Token),
    Queue(TokenQueue)
}

impl From<TokenQueue> for QueueItem {
    fn from(value: TokenQueue) -> Self {
        QueueItem::Queue(value)
    }
}

impl From<f64> for QueueItem {
    fn from(value: f64) -> Self {
        QueueItem::Number(value)
    }
}

impl From<Token> for QueueItem {
    fn from(value: Token) -> Self {
        QueueItem::Token(value)
    }
}

impl TokenQueue {
    fn push(&mut self, item: QueueItem) {
        self.0.push(item)
    }

    pub fn new(input: &str, variables: &[String]) -> Result<Self, ParseError> {
        let mut s = Self(Vec::new());
        
        let mut string_buff = String::new();
        let input = Self::add_parenthesis(&input.trim().replace(' ',""));

        let mut chars = input.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '(' {
                let mut buffer = String::new();
                let mut paren_count = 1;
                // TODO: Add error handling for if no parentheses
                for c in chars.by_ref() {
                    buffer.push(c);

                    if c == '(' {
                        paren_count += 1;
                    }
                    if c == ')' {
                        paren_count -= 1;
                        if paren_count == 0 {
                            break;
                        }
                    }
                }
                buffer.pop();
                s.push(Self::new(&buffer, variables)?.into());
            }
            if let Some(number) = Self::get_next_number(&mut chars, &c) {
                s.push(number.into())
            }
            if let Some(op) = Token::new(&c.to_string()) {
                s.push(op.into());
            }
            else if c.is_alphabetic() {
                dbg!(c);
                string_buff.push(c);
                while variables.iter().any(|var|var.starts_with(&string_buff)) {
                    if let Some(op) = Token::new(&string_buff) {
                        s.push(op.into());
                        string_buff.clear();
                        break;
                    } else if variables.iter().any(|var|var.eq(&string_buff)) {
                        s.push(QueueItem::Variable(string_buff.clone()));
                        dbg!(&string_buff);
                        break;
                    } else {
                        dbg!(&string_buff);
                        string_buff.push(chars.next().ok_or(ParseError::UnableToFind(format!("Variable {}", string_buff)))?);
                        dbg!(&string_buff);
                    }
                }
                string_buff.clear();
                
                
            }
        }
        Ok(s)
    }

    fn add_parenthesis(input: &str) -> String {
        if !(input.contains('+') || input.contains('-')) {
            return input.to_string();
        }
        let re = Regex::new(r#"((\d+\s*[*/])+\d+)"#).unwrap();
        re.replace_all(input,"($1)").to_string()
    }

    fn get_next_number(chars: &mut Peekable<Chars>, current_char: &char) -> Option<f64> {
        if current_char.is_ascii_digit() {
            let mut num_buffer = String::from(current_char.to_owned());
            while chars.peek().is_some_and(|c|c.is_ascii_digit()||c==&'.') {
                num_buffer.push(chars.next().unwrap())
            }
            Some(num_buffer.parse().unwrap())
        } else {
            None
        }
    }
    pub fn calculate(&self, var_map: &HashMap<String,f64>) -> Result<f64, ParseError> {
        let mut previous_num = 0.;
        let mut list = self.0.iter().peekable();

        
        while let Some(item) = list.next() {
            match item {
                QueueItem::Variable(var_name) => {
                    let var = var_map.get(var_name).ok_or(ParseError::UnableToFind(format!("variable \"{}\"",var_name)))?;
                    if previous_num == 0. {
                        previous_num = *var;
                    } else {
                        previous_num *= var
                    }
                }
                QueueItem::Number(num) => {
                    previous_num = *num
                }
                QueueItem::Token(token) => {
                    match list.next(){
                        Some(item) => match item {
                            QueueItem::Token(..) => {
                                return Err(ParseError::InvalidTokenPosition);
                            },
                            QueueItem::Variable(var_name) => {
                                previous_num = token.to_operation(previous_num, *var_map.get(var_name).ok_or(ParseError::UnableToFind(format!("variable \"{}\"",var_name)))?).do_operation();
                            }
                            QueueItem::Number(num2) => {
                                previous_num = token.to_operation(previous_num, *num2).do_operation();
                            }
                            QueueItem::Queue(inner_queue) => {
                                previous_num = token.to_operation(previous_num, inner_queue.calculate(var_map)?).do_operation()
                            }
                        }
                        None => Err(ParseError::UnableToParse)?
                    }
                    }
                
                QueueItem::Queue(q) => previous_num = q.calculate(var_map)?
                
            }
        }
        Ok(previous_num)
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use crate::parse::TokenQueue;

    #[test]
    fn test_add_parenthesis() {
        assert_eq!(super::TokenQueue::add_parenthesis("1*1"), "(1*1)");
        assert_eq!(super::TokenQueue::add_parenthesis("1/1"), "(1/1)");
        assert_eq!(super::TokenQueue::add_parenthesis("1*1*1"), "(1*1*1)");
        assert_eq!(super::TokenQueue::add_parenthesis("1*1/20+12-17*20"), "(1*1/20)+12-(17*20)");
    }

    #[test]
    fn test_queue() {
        let q = TokenQueue::new("t^2", &["t".to_string()]).unwrap();
        dbg!(&q);
        dbg!(q.calculate(&HashMap::from([("t".to_string(),1.)])));
    }
}