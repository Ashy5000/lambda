use std::thread;
use std::time::Duration;
use speedy2d::Graphics2D;
use speedy2d::color::Color;
use speedy2d::font::{Font, TextAlignment, TextLayout, TextOptions};
use speedy2d::window::{WindowHandler, WindowHelper};
use crate::expr::LambdaExpr;

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
        let term_string = if self.terms.len() > 1 {
            self.terms.remove(0).to_string()
        } else {
            if self.res.len() > 0 {
                self.res_cmp.push(self.res.remove(0));
            }
            self.terms[0].to_string() + self.res_cmp.as_str()
        };
        let term_str = term_string.as_str();
        let text_options: TextOptions = TextOptions::new().with_wrap_to_width(300.0, TextAlignment::Left);
        let text = self.font.layout_text(term_str, 13.0, text_options);
        graphics.draw_text((100.0, 100.0), Color::WHITE, &text);
        helper.request_redraw();
    }
}