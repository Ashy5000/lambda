use std::collections::HashMap;
use crate::expr::{ExprType, LambdaExpr};

const PADDING: f32 = 10.0;

pub(crate) enum Direction {
    Vertical,
    Horizontal
}

pub(crate) struct Line {
    pub(crate) origin: (f32, f32),
    pub(crate) length: f32,
    pub(crate) direction: Direction
}

impl Line {
    fn endpoint(&self) -> (f32, f32) {
        match self.direction {
            Direction::Vertical => (self.origin.0, self.origin.1 + self.length),
            Direction::Horizontal => (self.origin.0 + self.length, self.origin.1)
        }
    }
}

pub(crate) struct Diagram {
    pub(crate) lines: Vec<Line>
}

impl Diagram {
    pub(crate) fn rightmost(&self) -> (f32, f32) {
        let mut largest_x = 0.0;
        let mut y = 0.0;
        for line in &self.lines {
            if line.endpoint().0 > largest_x {
                largest_x = line.endpoint().0;
                y = line.endpoint().1;
            }
        }
        (largest_x, y)
    }

    pub(crate) fn bottommost(&self) -> (f32, f32) {
        let mut largest_y = 0.0;
        let mut x = 0.0;
        for line in &self.lines {
            if line.endpoint().1 > largest_y {
                largest_y = line.endpoint().1;
                x = line.endpoint().0;
            }
        }
        (x, largest_y)
    }

    fn shift(&mut self, vec: (f32, f32)) {
        for line in &mut self.lines {
            line.origin.0 += vec.0;
            line.origin.1 += vec.1;
        }
    }

    fn merge(&mut self, other: Diagram) {
        self.lines.extend(other.lines);
    }

    fn add_line(&mut self, line: Line) {
        self.lines.push(line);
    }

    fn add_lines(&mut self, lines: Vec<Line>) {
        for line in lines {
            self.add_line(line);
        }
    }
}

#[derive(Clone)]
pub(crate) struct Passthrough {
    var_positions: HashMap<usize, f32>,
    next_position: f32,
}

impl Passthrough {
    pub(crate) fn top() -> Self {
        Self {
            var_positions: HashMap::default(),
            next_position: 0.0,
        }
    }
}

pub(crate) fn construct_diagram(expr: &LambdaExpr, p: &Passthrough) -> Diagram {
    match expr.expr_type {
        ExprType::Var => {
            Diagram{
                lines: vec![Line{
                    origin: (PADDING, p.var_positions[&expr.id]),
                    length: p.next_position - p.var_positions[&expr.id],
                    direction: Direction::Vertical
                }]
            }
        }
        ExprType::Abs => {
            let mut new_p = p.clone();
            new_p.var_positions.insert(expr.id, p.next_position);
            new_p.next_position += PADDING;
            let mut child = construct_diagram(&expr.children[0], &new_p);
            let var_bar = Line{
                origin: (0.0, p.next_position),
                length: child.rightmost().0 + if expr.children[0].expr_type == ExprType::Abs { 0.0 } else { PADDING },
                direction: Direction::Horizontal
            };
            child.add_line(var_bar);
            child
        }
        ExprType::App => {
            let mut a = construct_diagram(&expr.children[0], p);
            let mut b = construct_diagram(&expr.children[1], p);
            b.shift((a.rightmost().0 + PADDING, 0.0));
            let point_a = a.bottommost();
            let point_b = b.bottommost();
            let cross_y = point_a.1.max(point_b.1) + PADDING;
            a.merge(b);
            let line_a = Line{
                origin: point_a,
                length: cross_y - point_a.1 + PADDING,
                direction: Direction::Vertical
            };
            let line_b = Line{
                origin: point_b,
                length: cross_y - point_b.1,
                direction: Direction::Vertical
            };
            let line_cross = Line{
                origin: (point_a.0, cross_y),
                length: point_b.0 - point_a.0,
                direction: Direction::Horizontal
            };
            a.add_lines(vec![line_a, line_b, line_cross]);
            a
        }
    }
}