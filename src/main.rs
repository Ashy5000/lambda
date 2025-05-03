mod ollama;
mod expr;
mod reduction;
mod decoding;
mod numerals;
mod graphics;
mod diagrams;
mod sound;

use std::io;
use speedy2d::window::WindowCreationOptions;
use crate::ollama::{handle_prompt, instantiate_ollama};

#[tokio::main]
async fn main() {
    let mut ollama = instantiate_ollama();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let options = WindowCreationOptions::new_fullscreen_borderless();
    let window = speedy2d::Window::new_with_options("Lambda", options).unwrap();
    handle_prompt(input, &mut ollama, window).await;
}