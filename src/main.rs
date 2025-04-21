use std::cmp::PartialEq;
use std::io;
use std::fmt;
use std::fmt::Formatter;
use rand::distr::StandardUniform;
use rand::prelude::*;
use rand::rng;

const MIN_DIFFICULTY: i32 = 1;
const MAX_DIFFICULTY: i32 = 10;

#[derive(PartialEq, Clone, Debug)]
enum ExprType {
    Var,
    Abs,
    App,
}

#[derive(Clone, Debug)]
struct LambdaExpr {
    expr_type: ExprType,
    id: usize,
    children: Vec<LambdaExpr>,
}

impl Distribution<ExprType> for StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> ExprType {
        let range: Vec<i32> = (0..=2).collect();
        match range.choose(rng) {
            None => panic!("RNG failed."),
            Some(x) => match *x {
                0 => ExprType::Var,
                1 => ExprType::Abs,
                2 => ExprType::App,
                _ => panic!("RNG failed.")
            }
        }
    }
}

impl fmt::Display for LambdaExpr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.expr_type {
            ExprType::Var => write!(f, "{}", self.id as u8 as char),
            ExprType::App => write!(f, "({})({})", self.children[0], self.children[1]),
            ExprType::Abs => write!(f, "λ{}.{}", self.id as u8 as char, self.children[0])
        }
    }
}

fn random_expr<R:Rng>(rng: &mut R, vars: Vec<usize>) -> LambdaExpr {
    let mut expr_type: ExprType = rng.random();
    if expr_type == ExprType::Var && vars.is_empty() {
        expr_type = ExprType::Abs;
    }
    let range: Vec<usize> = (97..=122).collect();
    match expr_type {
        ExprType::Var => LambdaExpr{
            expr_type,
            id: *vars.choose(rng).unwrap(),
            children: vec![]
        },
        ExprType::Abs => {
            let id = *range.choose(rng).unwrap();
            let mut new_vars = vars.clone();
            new_vars.push(id);
            LambdaExpr {
                expr_type,
                id,
                children: vec![
                    random_expr(rng, new_vars)
                ]
            }
        },
        ExprType::App => LambdaExpr {
            expr_type,
            id: 0,
            children: vec![
                random_expr(rng, vars.clone()),
                random_expr(rng, vars),
            ]
        }
    }
}

fn substitute(expr: &mut LambdaExpr, from: usize, to: LambdaExpr) {
    if expr.id == from {
        if expr.expr_type == ExprType::Var {
            expr.expr_type = to.expr_type;
            expr.id = to.id;
            expr.children = to.children;
        } else if expr.expr_type == ExprType::Abs {
            expr.id += 1;
            substitute(expr, from, LambdaExpr{
                expr_type: ExprType::Var,
                id: from + 1,
                children: vec![],
            });
        }
    } else {
        for child in &mut expr.children {
            substitute(child, from, to.clone());
        }
    }
}

fn beta_reduce_step(expr: &mut LambdaExpr) -> bool {
    if expr.expr_type == ExprType::App && expr.children[0].expr_type == ExprType::Abs {
        expr.expr_type = expr.children[0].children[0].expr_type.clone();
        expr.id = expr.children[0].children[0].id;
        let from_tmp = expr.children[0].id;
        let to_tmp = expr.children[1].clone();
        expr.children = expr.children[0].children[0].children.clone();
        substitute(expr, from_tmp, to_tmp);
        true
    } else {
        let mut found_redex = false;
        for child in &mut expr.children {
            found_redex |= beta_reduce_step(child);
        }
        found_redex
    }
}

fn beta_reduce(e: &LambdaExpr) -> (i32, LambdaExpr) {
    let mut i = 0;
    let mut expr = e.clone();
    while beta_reduce_step(&mut expr) && i < MAX_DIFFICULTY { i += 1 }
    (if beta_reduce_step(&mut expr) {
        11
    } else {
        i
    }, expr)
}

fn interpret_expr(input: &String) -> Option<LambdaExpr> {
    if input.matches('(').count() != input.matches(')').count() {
        return None;
    }
    let mut input_clone = input.clone();
    if input.chars().nth(0)? == '(' {
        let mut depth = 0;
        let mut can_remove = true;
        let mut input_truncated = input.clone();
        input_truncated.pop();
        for c in input_truncated.chars() {
            match c {
                '(' => depth += 1,
                ')' => depth -= 1,
                _ => {}
            }
            if depth == 0 {
                can_remove = false;
                break;
            }
        }
        if can_remove {
            input_clone.pop();
            input_clone.remove(0);
        }
    }
    let chars = input_clone.chars();
    if input.len() == 1 {
        Some(LambdaExpr {
            expr_type: ExprType::Var,
            id: input.chars().nth(0)? as usize,
            children: vec![],
        })
    } else if chars.clone().nth(0)? == 'λ' {
        Some(LambdaExpr {
            expr_type: ExprType::Abs,
            id: chars.clone().nth(1)? as usize,
            children: vec![
                interpret_expr(&chars.clone().skip(3).take_while(|_| true).collect())?
            ]
        })
    } else {
        let mut a = input_clone.clone();
        let mut b = String::new();
        for c in input_clone.chars().rev() {
            b.insert(0, c);
            a.pop();
            if interpret_expr(&b).is_some() {
                break;
            }
        }
        Some(LambdaExpr {
            expr_type: ExprType::App,
            id: 0,
            children: vec![
                interpret_expr(&a)?,
                interpret_expr(&b)?
            ]
        })
    }
}

impl PartialEq for LambdaExpr {
    fn eq(&self, other: &Self) -> bool {
        if self.expr_type != other.expr_type {
            false
        } else if self.id != other.id {
            false
        } else if self.children.len() != other.children.len() {
            false
        } else {
            for i in 0..self.children.len() {
                if self.children[i] != other.children[i] {
                    return false
                }
            }
            true
        }
    }
}

fn main() {
    let mut rng = rng();
    let mut diff = MAX_DIFFICULTY + 1;
    let mut generated = LambdaExpr{
        expr_type: ExprType::Var,
        id: 0,
        children: vec![]
    };
    let mut normal_form = LambdaExpr{
        expr_type: ExprType::Var,
        id: 0,
        children: vec![]
    };
    while diff < MIN_DIFFICULTY || diff > MAX_DIFFICULTY {
        generated = random_expr(&mut rng, vec![]);

        (diff, normal_form) = beta_reduce(&mut generated);
    }
    println!("{generated}");
    println!("Difficulty: {}", diff);
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input = input.replace("L", "λ");
    let guess = interpret_expr(&String::from(input.trim())).unwrap();
    if guess == normal_form {
        println!("Correct.");
    } else {
        println!("Incorrect. Expected: {}", normal_form)
    }
}