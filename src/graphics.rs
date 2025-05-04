use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;
use speedy2d::Graphics2D;
use speedy2d::color::Color;
use speedy2d::font::{Font, TextAlignment, TextLayout, TextOptions};
use speedy2d::window::{KeyScancode, VirtualKeyCode, WindowHandler, WindowHelper};
use crate::diagrams::{construct_diagram, Direction, Passthrough};
use crate::expr::LambdaExpr;
use crate::numerals::unchurch;
use crate::ollama::{handle_prompt, instantiate_ollama};
use crate::sound::sound_thread;

const LINE_THICKNESS: f32 = 5.0;
const TEXT_SIZE: f32 = 13.0;
const TEXT_CUTOFF: usize = 3000;
const TEXT_WIDTH: f32 = 300.0;
const TEXT_PADDING: f32 = 100.0;
const DELAY: u64 = 10;
const DELAY_START_MULTIPLIER: u64 = 1;

pub(crate) struct LambdaGraphicsHandler {
    pub(crate) terms: Vec<LambdaExpr>,
    pub(crate) font: Font,
    pub(crate) res: String,
    res_cmp: String,
    delay: u64,
    original_terms: Vec<LambdaExpr>,
    original_res: String,
    first_frame: bool,
    played_sound: bool,
    play_next_frame: bool,
    trigger_flag: Arc<Mutex<bool>>,
    frames_to_render: i64,
    frame: u64,
    prompt: String
}

impl LambdaGraphicsHandler {
    pub(crate) fn new(font: Font) -> Self {
        Self {
            terms: vec![],
            font,
            res: String::new(),
            res_cmp: String::new(),
            delay: DELAY * DELAY_START_MULTIPLIER,
            original_terms: vec![],
            original_res: String::new(),
            first_frame: true,
            played_sound: false,
            play_next_frame: false,
            trigger_flag: sound_thread(),
            frames_to_render: -1,
            frame: 1,
            prompt: String::new()
        }
    }
}

impl WindowHandler for LambdaGraphicsHandler {
    fn on_draw(&mut self, helper: &mut WindowHelper<()>, graphics: &mut Graphics2D) {
        let win_size = helper.get_size_pixels();
        if self.terms.len() == 0 {
            graphics.clear_screen(Color::BLACK);
            let text_options: TextOptions = TextOptions::new().with_wrap_to_width(TEXT_WIDTH, TextAlignment::Center);
            let text = self.font.layout_text(self.prompt.as_str(), TEXT_SIZE, text_options);
            graphics.draw_text((win_size.x as f32 / 2.0 - (TEXT_WIDTH / 2.0), win_size.y as f32 / 2.0), Color::WHITE, &text);
            helper.request_redraw();
            return;
        }
        if self.frames_to_render == 0 {
            helper.request_redraw();
            return;
        }
        self.frames_to_render -= 1;
        if self.frame % self.delay != 0 {
            self.frame += 1;
            helper.request_redraw();
            return;
        }
        self.frame += 1;
        graphics.clear_screen(Color::BLACK);
        if self.first_frame {
            self.first_frame = false;
            helper.request_redraw();
            return;
        }
        let (term, removed) = if self.terms.len() > 1 {
            (self.terms.remove(0), true)
        } else {
            (self.terms[0].clone(), false)
        };
        let diagram = construct_diagram(&term, &Passthrough::top());
        let right_edge = diagram.rightmost().0;
        let bottom_edge = diagram.bottommost().1;
        let x_scale = (win_size.x as f32 - (TEXT_WIDTH + 2.0 * TEXT_PADDING)) / right_edge;
        let y_scale = win_size.y as f32 / bottom_edge;
        let scale = x_scale.min(y_scale).min(1.0);
        let x_offset = (win_size.x as f32 - (TEXT_WIDTH + 2.0 * TEXT_PADDING) - (right_edge * scale)) / 2.0;
        let y_offset = (win_size.y as f32 - (bottom_edge * scale)) / 2.0;
        for line in diagram.lines {
            let startpoint = match line.direction {
                Direction::Vertical => (line.origin.0 * scale + x_offset, (line.origin.1 - (LINE_THICKNESS * scale) / 2.0) * scale + y_offset),
                Direction::Horizontal => (line.origin.0 * scale + x_offset, line.origin.1 * scale + y_offset)
            };
            let endpoint = match line.direction {
                Direction::Vertical => (line.origin.0 * scale + x_offset, (line.origin.1 + line.length + (LINE_THICKNESS * scale) / 2.0) * scale + y_offset),
                Direction::Horizontal => ((line.origin.0 + line.length + (LINE_THICKNESS * scale) / 2.0) * scale + x_offset, line.origin.1 * scale + y_offset)
            };
            graphics.draw_line(startpoint, endpoint, LINE_THICKNESS * scale, if removed { Color::WHITE } else { Color::BLUE });
        }
        if !removed && !self.played_sound {
            self.play_next_frame = true;
            self.played_sound = true;
        } else if self.play_next_frame {
            *Arc::clone(&self.trigger_flag).lock().unwrap() = true;
            self.play_next_frame = false;
        }
        let term_string: String = if removed {
            let string = term.to_string();
            if string.len() <= TEXT_CUTOFF {
                string
            } else {
                string.chars().take(TEXT_CUTOFF).collect::<String>() + "..."
            }
        } else {
            if self.res.len() > 0 {
                self.res_cmp.push(self.res.remove(0));
            }
            let string = term.to_string();
            (if string.len() <= TEXT_CUTOFF {
                string
            } else {
                string.chars().take(TEXT_CUTOFF).collect::<String>() + "..."
            }) + self.res_cmp.as_str()
        };
        let term_str = term_string.as_str();
        let text_options: TextOptions = TextOptions::new().with_wrap_to_width(TEXT_WIDTH, TextAlignment::Left);
        let text = self.font.layout_text(term_str, TEXT_SIZE, text_options);
        graphics.draw_text((win_size.x as f32 - (TEXT_WIDTH + TEXT_PADDING), TEXT_PADDING), Color::WHITE, &text);
        helper.request_redraw();
    }

    fn on_key_down(&mut self, helper: &mut WindowHelper<()>, virtual_key_code: Option<VirtualKeyCode>, _: KeyScancode) {
        let key_code = match virtual_key_code {
            Some(x) => x,
            None => { return }
        };
        if self.original_terms.len() == 0 {
            if key_code == VirtualKeyCode::Return {
                let mut ollama = instantiate_ollama();
                self.original_terms = futures::executor::block_on(handle_prompt(self.prompt.clone(), &mut ollama));
                self.terms = self.original_terms.clone();
                self.res = format!(" = {}", unchurch(&self.terms[self.terms.len() - 1]));
                self.prompt = String::new();
                *Arc::clone(&self.trigger_flag).lock().unwrap() = true;
            } else if key_code == VirtualKeyCode::Backspace {
                self.prompt.pop();
            }
            return;
        }
        if key_code == VirtualKeyCode::Return {
            if self.terms.len() == 1 {
                *Arc::clone(&self.trigger_flag).lock().unwrap() = true;
            }
            self.terms = self.original_terms.clone();
            self.res = self.original_res.clone();
            self.res_cmp = String::new();
            self.delay = DELAY * DELAY_START_MULTIPLIER;
            self.played_sound = false;
            self.first_frame = true;
            self.frames_to_render = -1;
            self.frame = 0;
            helper.request_redraw();
        } else if key_code == VirtualKeyCode::Space {
            if self.frames_to_render != 0 {
                self.frames_to_render = 0;
            } else {
                self.frames_to_render = -1;
            }
            self.frame = 0;
            helper.request_redraw();
        } else if key_code == VirtualKeyCode::Left {
            if self.original_terms.len() - 2 < self.terms.len() {
                return;
            }
            self.terms.insert(0, self.original_terms[self.original_terms.len() - self.terms.len() - 1].clone());
            self.terms.insert(0, self.original_terms[self.original_terms.len() - self.terms.len() - 1].clone());
            if self.frames_to_render <= 0 {
                self.frames_to_render = 1;
            }
            self.frame = 0;
            helper.request_redraw();
        } else if key_code == VirtualKeyCode::Right {
            if self.terms.len() <= 1 {
                return;
            }
            if self.frames_to_render <= 0 {
                self.frames_to_render = 1;
            }
            self.frame = 0;
            helper.request_redraw();
        } else if key_code == VirtualKeyCode::Tab {
            if self.terms.len() > 1 {
                *Arc::clone(&self.trigger_flag).lock().unwrap() = true;
                sleep(Duration::from_millis(20));
                *Arc::clone(&self.trigger_flag).lock().unwrap() = true;
            }
            self.terms = vec![];
            self.original_terms = vec![];
            self.res = String::new();
            self.res_cmp = String::new();
            self.prompt = String::new();
            self.frame = 0;
            self.played_sound = false;
            self.first_frame = true;
            self.frames_to_render = -1;
            helper.request_redraw();
        }
    }

    fn on_keyboard_char(&mut self, _: &mut WindowHelper<()>, unicode_codepoint: char) {
        if unicode_codepoint.is_control() {
            return;
        }
        if self.original_terms.len() == 0 {
            self.prompt.push(unicode_codepoint);
        }
    }
}