use std::ascii::Char;
use std::collections::VecDeque;
use std::iter::Peekable;
use std::str::Chars;
use itertools::Itertools;
use prse::{Parse, try_parse};
use regex::Regex;
use log::warn;

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


    fn into_operation(self, num1:f64,num2: f64) -> Box<dyn Operation> {
        let (n1,n2) = (num1,num2);
        match self {
            Self::Add => Box::new(Add(n1,n2)),
            Self::Subtract => Box::new(Subtract(n1,n2)),
            Self::Multiply => Box::new(Multiply(n1,n2)),
            Self::Divide => Box::new(Divide(n1,n2)),
            Self::TestFunction{a,b} => Box::new(TestFunction(a,b))
        }
    }


}

#[derive(Debug, Clone)]
pub struct TokenQueue(Vec<QueueItem>);

#[derive(Debug, Clone)]
enum QueueItem {
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
    pub fn push(&mut self, item: QueueItem) {
        self.0.push(item)
    }

    pub fn new(input: &str) -> Self {
        let mut s = Self(Vec::new());
        
        let mut operation_buff = String::new();
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
                s.push(Self::new(&buffer).into());
            }
            if let Some(number) = Self::get_next_number(&mut chars, &c) {
                s.push(number.into())
            }
            match Token::new(c.to_string().as_str()) {
                Some(op) => {
                    s.push(op.into())
                },
                None => {
                    operation_buff.push(c);
                    if let Some(op) = Token::new(&operation_buff) {
                        s.push(op.into());
                        operation_buff.clear();
                    }
                }
            }
        }
        s
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
    pub fn calculate(self) -> Option<f64> {
        let mut previous_num = 0.;
        let mut list = self.0.into_iter().peekable();

        while let Some(item) = list.next() {
            match item {
                QueueItem::Number(num) => {
                    previous_num = num
                }
                QueueItem::Token(t) => {
                    match list.next() {
                        Some(QueueItem::Token(..)) => {
                            warn!("error here");
                            return None;
                        },
                        Some(QueueItem::Number(num2)) => {
                            previous_num = t.into_operation(previous_num,num2).do_operation();
                        }
                        Some(QueueItem::Queue(q)) => {
                            if let Some(num) = q.calculate() {
                                previous_num = t.into_operation(previous_num,num).do_operation()
                            } else {
                                warn!("error!");
                                return None;
                            }
                        }
                        None => ()
                    }
                }
                QueueItem::Queue(q) => {
                    if let Some(num) = q.calculate() {
                        previous_num = num
                    } else {
                        warn!("error here!");
                        return None;
                    }
                }
            }
        }
        Some(previous_num)
    }
}

#[cfg(test)]
mod test {
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
        let q = TokenQueue::new("12+20*10");
        dbg!(&q);
        dbg!(q.calculate());
    }
}