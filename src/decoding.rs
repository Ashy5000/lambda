use std::fmt::format;
use crate::expr::{ExprType, LambdaExpr};
use crate::numerals::church;

pub(crate) fn interpret_expr(input: &String) -> Option<LambdaExpr> {
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
    if input_clone.len() == 1 {
        Some(LambdaExpr {
            expr_type: ExprType::Var,
            id: input_clone.chars().nth(0)? as usize,
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

pub(crate) fn arithmetic_to_lambda(input: &String) -> LambdaExpr {
    let mut words: Vec<String> = input.split(" ").map(|x| x.to_string()).collect();
    if words.len() == 1 {
        church(words[0].parse().unwrap())
    } else {
        let add = interpret_expr(&"λm.λn.λf.λx.mf(nfx)".to_string()).unwrap();
        let mul = interpret_expr(&"λm.λn.λf.λx.m(nf)x".to_string()).unwrap();
        let pred = interpret_expr(&"λn.λf.λx.n(λg.λh.h(gf))(λu.x)(λu.u)".to_string()).unwrap();
        let sub = interpret_expr(&format!("λm.λn.n({pred})m")).unwrap();
        let div = interpret_expr(&"(λn.((λf.(λx.xx)(λx.f(xx)))(λc.λn.λm.λf.λx.(λd.(λn.n(λx.(λa.λb.b))(λa.λb.a))d((λf.λx.x)fx)(f(cdmfx)))((λm.λn.n(λn.λf.λx.n(λg.λh.h(gf))(λu.x)(λu.u))m)nm)))((λn.λf.λx.f(nfx))n))".to_string()).unwrap();
        let iszero = interpret_expr(&"λn.n(λx.(λa.λb.b))(λa.λb.a)".to_string()).unwrap();
        let fac = interpret_expr(&format!("(λx.(λy.x(yy))(λy.x(yy)))(λf.λn.(({iszero})n)({})(({mul})n(f(({pred})n))))", church(1))).unwrap();
        let first_word = words.pop().unwrap();
        let (x, op_string) = if first_word != "!" {
            (first_word.parse().unwrap(), words.pop().unwrap())
        } else {
            (words.pop().unwrap().parse().unwrap(), first_word)
        };
        let op = op_string.as_str();
        let op_lambda = match op {
            "+" => add,
            "-" => sub,
            "*" => mul,
            "/" => div,
            "!" => fac,
            x => panic!("Unknown operation {x}")
        };
        let x_lambda = church(x);
        LambdaExpr {
            expr_type: ExprType::App,
            id: 0,
            children: vec![
                if op_string == "!" {
                    op_lambda
                } else {
                    LambdaExpr {
                        expr_type: ExprType::App,
                        id: 0,
                        children:
                        vec![
                            op_lambda,
                            arithmetic_to_lambda(&words.join(" "))
                        ],
                    }
                },
                x_lambda
            ]
        }
    }
}