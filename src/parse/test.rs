use crate::parse::token::{Token, TokenQueue};


fn op(input: &str) -> TokenQueue {
    TokenQueue::new(input)
}

#[test]
fn add() {
    dbg!(op("1+1"));
}
