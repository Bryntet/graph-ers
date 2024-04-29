use crate::parse::math_functions::ParseError;
use itertools::Itertools;
use log::warn;
use prse::{try_parse, Parse};
use regex::Regex;
use std::ascii::Char;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::iter::Peekable;
use std::str::Chars;

pub(crate) trait Operation {
    fn do_operation(&self) -> f64;
    fn required_args(&self) -> usize {
        2
    }

    fn operation_type(&self) -> Token;
}

struct Add(f64, f64);

impl Operation for Add {
    fn do_operation(&self) -> f64 {
        self.0 + self.1
    }
    fn operation_type(&self) -> Token {
        Token::Add
    }
}
struct Subtract(f64, f64);
impl Operation for Subtract {
    fn do_operation(&self) -> f64 {
        self.0 - self.1
    }

    fn operation_type(&self) -> Token {
        Token::Subtract
    }
}

struct Multiply(f64, f64);
impl Operation for Multiply {
    fn do_operation(&self) -> f64 {
        self.0 * self.1
    }

    fn operation_type(&self) -> Token {
        Token::Multiply
    }
}

struct Divide(f64, f64);

impl Operation for Divide {
    fn do_operation(&self) -> f64 {
        self.0 / self.1
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

struct TestFunction(f64, f64);
impl Operation for TestFunction {
    fn do_operation(&self) -> f64 {
        self.0 * self.1 / 2.
    }

    fn operation_type(&self) -> Token {
        Token::TestFunction {
            a: self.0,
            b: self.1,
        }
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
    TestFunction { a: f64, b: f64 },
}
struct Function(FunctionType);
impl Function {
    fn test(&self) {
        println!("Test")
    }
}
enum FunctionType {
    Test,
}

impl Token {
    fn new(input: &str) -> Option<Self> {
        let input = input.replace(' ', "");
        try_parse!(&input, "{}").ok()
    }

    fn to_operation(&self, num1: f64, num2: f64) -> Box<dyn Operation> {
        let (n1, n2) = (num1, num2);
        match self {
            Self::Add => Box::new(Add(n1, n2)),
            Self::Subtract => Box::new(Subtract(n1, n2)),
            Self::Multiply => Box::new(Multiply(n1, n2)),
            Self::Divide => Box::new(Divide(n1, n2)),
            Self::Pow => Box::new(Pow(n1, n2)),
            Self::TestFunction { a, b } => Box::new(TestFunction(*a, *b)),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenQueue{
    queue_items: Vec<QueueItem>,
    pub input_representation: String
}

#[derive(Debug, Clone, PartialEq)]
enum QueueItem {
    Variable(String),
    Number(f64),
    Token(Token),
    Queue(TokenQueue),
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
        self.queue_items.push(item)
    }

    pub fn new(input: &str, variables: &[String]) -> Result<Self, ParseError> {
        let input
 = input.trim().replace(' ',"").to_lowercase();
        let input = Self::add_parenthesis(&input);
        let mut s = Self{queue_items:Vec::new(),input_representation: input.clone()};

        let mut string_buff = String::new();
        
        
        let mut chars = input.chars().peekable();
        while let Some(mut c) = chars.next() {
            if c == '(' {
                let mut buffer = String::new();

                let mut paren_count = 0;
                // TODO: Add error handling for if no parentheses
                while buffer.chars().filter(|b_c|b_c==&'(').count()+1 > paren_count {
                    if let Some(buff_char) = chars.next() {
                        buffer.push(buff_char);
                        if buff_char == ')' {
                            paren_count += 1;
                        }
                    } else {
                        return Err(ParseError::UnclosedParenthesis);
                    }
                    
                }
                buffer.pop();
                s.push(Self::new(&buffer, variables)?.into());
                if let Some(a) = chars.next() {
                    c=a
                }
            }
            if let Some(number) = Self::get_next_number(&mut chars, c)? {
                s.push(number.into())
            }
            if let Some(op) = Token::new(&c.to_string()) {
                s.push(op.into());
            } else if c.is_alphabetic() {
                string_buff.push(c);
                while variables.iter().any(|var| var.starts_with(&string_buff)) {
                    if let Some(op) = Token::new(&string_buff) {
                        s.push(op.into());
                        string_buff.clear();
                        break;
                    } else if variables.iter().any(|var| var.eq(&string_buff)) {
                        s.push(QueueItem::Variable(string_buff.clone()));
                        break;
                    } else {
                        string_buff.push(chars.next().ok_or(ParseError::UnableToFind(format!(
                            "Variable {}",
                            string_buff
                        )))?);
                    }
                }
                string_buff.clear();
            }
        }
        Ok(s)
    }
    fn add_parenthesis(input: &str) -> String {
        Self::remove_parenthesis(&Self::add_most_basic_parenthesis(&Self::add_parenthesis_multiplication(&Self::add_parenthesis_exponent(input))))
    }

    fn remove_parenthesis(input: &str) -> String {
        let re = Regex::new(r#"\((\d+)\)"#).expect("Regex is valid");
        re.replace_all(input, "$1").to_string()
    }

    /*fn add_hidden_multiplication(input: &str) -> String {
        let re = Regex::new(r#"(\d)(\(|[a-z])"#).expect("Regex is valid");
        let re2 = Regex::new(r#"([a-z])(\(|\d)"#).expect("Regex is valid");
        re2.replace_all(&re.replace_all(input,"$1*$2"), "$1*$2").to_string()
    }*/

    fn add_parenthesis_exponent(input: &str) -> String {
        if !input.contains(['+','-','*','/']) {
           input.to_string()
        } else {
            let re = Regex::new(r#"((?:(\d+)|([a-z]+))\^(\d*[a-z]*|\([^)]+\)))"#).expect("Regex is valid");
            re.replace_all(input, "($3^($4))").to_string()
        }
    }

    fn add_parenthesis_multiplication(input: &str) -> String {
        if !input.contains(['+','-']) {
            input.to_string()
        } else {
            let re = Regex::new(r#"((\(.+\)|\d*[a-z]*\s*)[*/](\(.+\)|\d*[a-z]*))"#).unwrap();
            re.replace_all(input, "($1)").to_string()
        }
    }

    fn add_most_basic_parenthesis(input: &str) -> String {
        let re = Regex::new(r#"([-+])([^()+\-]+)"#).expect("Regex is valid");
        re.replace_all(input, "$1($2)").to_string()
    }

    fn get_next_number(chars: &mut Peekable<Chars>, current_char: char) -> Result<Option<f64>,ParseError> {
        if current_char.is_ascii_digit() {
            let mut has_gone_decimal = false;
            let mut num_buffer = String::from(current_char.to_owned());
            while chars
                .peek()
                .is_some_and(|c|
                    c.is_ascii_digit() || c == &'.'
                )
            {
                let c = chars.next().expect("Already peeked forward");
                if c == '.' {
                    if has_gone_decimal {
                        return Err(ParseError::DoubleDecimal);
                    }
                    has_gone_decimal = true;
                }
                dbg!(num_buffer.push(c))
            }
            Ok(Some(num_buffer.parse().unwrap()))
        } else {
            Ok(None)
        }
    }
    fn get_var_value(var_name: &str, var_map: &HashMap<String,f64>) -> Result<f64,ParseError> {
        var_map.get(var_name).copied().ok_or(ParseError::UnableToFind(format!("variable: \"{}\"",var_name)))
    }

    pub fn calculate(&self, var_map: &HashMap<String, f64>) -> Result<f64, ParseError> {
        let previous_num: &mut Option<f64> = &mut None;
        let mut list = self.queue_items.iter().peekable();
        while let Some(item) = list.next() {
            match item {
                QueueItem::Variable(var_name) => {
                    let mut var = Self::get_var_value(var_name,var_map)?;
                    while let Some(QueueItem::Variable(_)|QueueItem::Number(_)) = list.peek() {
                        match list.next() {
                            Some(QueueItem::Variable(var_name)) => var *= Self::get_var_value(var_name,var_map)?,
                            Some(QueueItem::Number(num)) => var *= num,
                            _ => unreachable!()
                        }
                    }
                    if let Some(num) = previous_num {
                        *num *= var;
                    } else {
                        *previous_num = Some(var)
                    }
                }
                QueueItem::Number(num) => {
                    if let Some(prev_num) = previous_num {
                        *prev_num *= *num
                    } else {
                        *previous_num = Some(*num)
                    }
                },
                QueueItem::Token(token) => {
                    match list.next() {
                        Some(item) => match item {
                            QueueItem::Token(..) => {
                                return Err(ParseError::InvalidTokenPosition);
                            }
                            QueueItem::Variable(var_name) => {
                                *previous_num = Some(token
                                    .to_operation(
                                        previous_num.ok_or(ParseError::InvalidTokenPosition)?,
                                        *var_map.get(var_name).ok_or(ParseError::UnableToFind(
                                            format!("variable \"{}\"", var_name),
                                        ))?,
                                    )
                                    .do_operation());
                            }
                            QueueItem::Number(num2) => {
                                *previous_num = Some(token.to_operation(previous_num.ok_or(ParseError::InvalidTokenPosition)?, *num2).do_operation());
                            }
                            QueueItem::Queue(inner_queue) => {
                                *previous_num = Some(token
                                    .to_operation(previous_num.ok_or(ParseError::InvalidTokenPosition)?, inner_queue.calculate(var_map)?)
                                    .do_operation())
                            }
                        },
                        None => return Err(ParseError::UnableToFind("next item".to_string())),
                    }
                }
                QueueItem::Queue(q) => {
                    if let Some(num) = previous_num {
                        *num *= q.calculate(var_map)?
                    } else {
                        *previous_num  = Some(q.calculate(var_map)?)
                    }
                },
            }
        }
        previous_num.ok_or(ParseError::UnableToParse)
    }
}

#[cfg(test)]
mod test {
    use crate::parse::{Function, TokenQueue};
    use std::collections::HashMap;

    #[test]
    fn test_add_parenthesis() {
        assert_eq!(super::TokenQueue::add_parenthesis_multiplication("1*1"), "(1*1)");
        assert_eq!(super::TokenQueue::add_parenthesis_multiplication("1/1"), "(1/1)");
        assert_eq!(super::TokenQueue::add_parenthesis_multiplication("1*1*1"), "(1*1*1)");
        assert_eq!(
            super::TokenQueue::add_parenthesis_multiplication("1*1/20+12-17*20"),
            "(1*1/20)+12-(17*20)"
        );
    }

    #[test]
    fn test_queue() {
        let q = TokenQueue::new("t^2", &["t".to_string()]).unwrap();
        dbg!(&q);
        dbg!(q.calculate(&HashMap::from([("t".to_string(), 1.)])));
    }

    #[test]
    fn test_things() {
        let func = TokenQueue::new("(5t^3+5)", &["t".to_string()]).unwrap();
        assert_eq!(func.calculate(&HashMap::<String,f64>::from([("t".to_string(),1.)])), Ok(10.));
    }
}
