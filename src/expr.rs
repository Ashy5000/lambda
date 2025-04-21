use std::fmt;
use std::fmt::Formatter;
use rand::distr::{Distribution, StandardUniform};
use rand::prelude::IndexedRandom;
use rand::Rng;

#[derive(PartialEq, Clone, Debug)]
pub(crate) enum ExprType {
    Var,
    Abs,
    App,
}

#[derive(Clone, Debug)]
pub(crate) struct LambdaExpr {
    pub(crate) expr_type: ExprType,
    pub(crate) id: usize,
    pub(crate) children: Vec<LambdaExpr>,
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

fn decode_id(id: usize) -> String {
    format!("{}{}", (id % 1000) as u8 as char, if id < 1000 { "".to_string() } else { ((id - (id % 1000)) / 1000).to_string() })
}

impl fmt::Display for LambdaExpr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.expr_type {
            ExprType::Var => write!(f, "{}", decode_id(self.id)),
            ExprType::App => write!(f, "({})({})", self.children[0], self.children[1]),
            ExprType::Abs => write!(f, "λ{}.{}", decode_id(self.id), self.children[0])
        }
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