use std::thread;
use std::time::Duration;
use speedy2d::Graphics2D;
use speedy2d::color::Color;
use speedy2d::font::{Font, TextAlignment, TextLayout, TextOptions};
use speedy2d::window::{WindowHandler, WindowHelper};
use crate::diagrams::{construct_diagram, Direction, Passthrough};
use crate::expr::LambdaExpr;

const LINE_THICKNESS: f32 = 5.0;

pub(crate) struct LambdaGraphicsHandler {
    pub(crate) terms: Vec<LambdaExpr>,
    pub(crate) font: Font,
    pub(crate) res: String,
    res_cmp: String
}

impl LambdaGraphicsHandler {
    pub(crate) fn new(terms: Vec<LambdaExpr>, font: Font, res: String) -> Self {
        Self {
            terms,
            font,
            res,
            res_cmp: String::new()
        }
    }
}

impl WindowHandler for LambdaGraphicsHandler {
    fn on_draw(&mut self, helper: &mut WindowHelper<()>, graphics: &mut Graphics2D) {
        graphics.clear_screen(Color::BLACK);
        let (term, removed) = if self.terms.len() > 1 {
            (self.terms.remove(0), true)
        } else {
            (self.terms[0].clone(), false)
        };

        let diagram = construct_diagram(&term, &Passthrough::top());
        let win_size = helper.get_size_pixels();
        let right_edge = diagram.rightmost().0;
        let bottom_edge = diagram.bottommost().1;
        let x_scale = (win_size.x as f32 - 500.0) / right_edge;
        let y_scale = win_size.y as f32 / bottom_edge;
        let scale = x_scale.min(y_scale).min(1.0);
        let x_offset = (win_size.x as f32 - 500.0 - (right_edge * scale)) / 2.0;
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
            graphics.draw_line(startpoint, endpoint, LINE_THICKNESS * scale, Color::WHITE);
        }
        let term_string: String = if removed {
            let string = term.to_string();
            if string.len() <= 3000 {
                string
            } else {
                string.chars().take(3000).collect::<String>() + "..."
            }
        } else {
            if self.res.len() > 0 {
                self.res_cmp.push(self.res.remove(0));
            }
            let string = term.to_string();
            (if string.len() <= 3000 {
                string
            } else {
                string.chars().take(3000).collect::<String>() + "..."
            }) + self.res_cmp.as_str()
        };
        let term_str = term_string.as_str();
        let text_options: TextOptions = TextOptions::new().with_wrap_to_width(300.0, TextAlignment::Left);
        let text = self.font.layout_text(term_str, 13.0, text_options);
        graphics.draw_text((win_size.x as f32 - 400.0, 100.0), Color::WHITE, &text);
        thread::sleep(Duration::from_millis(50));
        helper.request_redraw();
    }
}