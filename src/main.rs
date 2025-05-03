mod ollama;
mod expr;
mod reduction;
mod decoding;
mod numerals;
mod graphics;

use std::{io, thread};
use crate::graphics::LambdaGraphicsHandler;
use crate::ollama::{handle_prompt, instantiate_ollama};

#[tokio::main]
async fn main() {
    let mut ollama = instantiate_ollama();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let window = speedy2d::Window::new_centered("Lambda", (500, 500)).unwrap();
    handle_prompt(input, &mut ollama, window).await;
}