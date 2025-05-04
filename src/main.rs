mod ollama;
mod expr;
mod reduction;
mod decoding;
mod numerals;
mod graphics;
mod diagrams;
mod sound;
use speedy2d::font::Font;
use speedy2d::window::WindowCreationOptions;
use crate::graphics::LambdaGraphicsHandler;

#[tokio::main]
async fn main() {
    let options = WindowCreationOptions::new_fullscreen_borderless();
    let window = speedy2d::Window::new_with_options("Lambda", options).unwrap();
    window.run_loop(LambdaGraphicsHandler::new(Font::new(include_bytes!("../IosevkaTermSlabNerdFont-Medium.ttf")).unwrap()));
}