use crate::expr::{ExprType, LambdaExpr};

fn substitute(expr: &mut LambdaExpr, from: usize, to: LambdaExpr) {
    if expr.id == from {
        if expr.expr_type == ExprType::Var {
            expr.expr_type = to.expr_type;
            expr.id = to.id;
            expr.children = to.children;
        } else if expr.expr_type == ExprType::Abs {
            expr.id += 1000;
            substitute(expr, from, LambdaExpr{
                expr_type: ExprType::Var,
                id: from + 1000,
                children: vec![],
            });
        }
    } else {
        for child in &mut expr.children {
            substitute(child, from, to.clone());
        }
    }
}

pub(crate) fn beta_reduce_step(expr: &mut LambdaExpr) -> bool {
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

pub(crate) fn beta_reduce(e: &mut LambdaExpr) {
    while beta_reduce_step(e) {}
}