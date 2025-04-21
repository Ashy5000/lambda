mod ollama;
mod expr;
mod reduction;
mod decoding;
mod numerals;

use std::io;
use crate::ollama::{handle_prompt, instantiate_ollama};

#[tokio::main]
async fn main() {
    let mut ollama = instantiate_ollama();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    handle_prompt(input, &mut ollama).await;
}