use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use soloud::*;
use speedy2d::Graphics2D;
use speedy2d::color::Color;
use speedy2d::font::{Font, TextAlignment, TextLayout, TextOptions};
use speedy2d::window::{KeyScancode, VirtualKeyCode, WindowHandler, WindowHelper, WindowStartupInfo};
use crate::diagrams::{construct_diagram, Direction, Passthrough};
use crate::expr::LambdaExpr;
use crate::sound::sound_thread;

const LINE_THICKNESS: f32 = 5.0;
const TEXT_SIZE: f32 = 13.0;
const TEXT_CUTOFF: usize = 3000;
const TEXT_WIDTH: f32 = 300.0;
const TEXT_PADDING: f32 = 100.0;
const DELAY: u64 = 50;
const DELAY_START_MULTIPLIER: u64 = 20;

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
    trigger_flag: Arc<Mutex<bool>>
}

impl LambdaGraphicsHandler {
    pub(crate) fn new(terms: Vec<LambdaExpr>, font: Font, res: String) -> Self {
        Self {
            terms: terms.clone(),
            font,
            res: res.clone(),
            res_cmp: String::new(),
            delay: DELAY * DELAY_START_MULTIPLIER,
            original_terms: terms,
            original_res: res,
            first_frame: true,
            played_sound: false,
            play_next_frame: false,
            trigger_flag: sound_thread()
        }
    }
}

impl WindowHandler for LambdaGraphicsHandler {
    fn on_draw(&mut self, helper: &mut WindowHelper<()>, graphics: &mut Graphics2D) {
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
        let win_size = helper.get_size_pixels();
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
        if self.terms.len() < 10 {
            self.delay += DELAY;
        } else if self.delay > DELAY {
            self.delay -= DELAY;
        }
        thread::sleep(Duration::from_millis(self.delay));
        helper.request_redraw();
    }

    fn on_key_down(&mut self, helper: &mut WindowHelper<()>, virtual_key_code: Option<VirtualKeyCode>, _: KeyScancode) {
        if virtual_key_code.unwrap() == VirtualKeyCode::Space {
            self.terms = self.original_terms.clone();
            self.res = self.original_res.clone();
            self.res_cmp = String::new();
            self.delay = DELAY * DELAY_START_MULTIPLIER;
            self.played_sound = false;
            self.first_frame = true;
            *Arc::clone(&self.trigger_flag).lock().unwrap() = true;
            helper.request_redraw();
        }
    }
}