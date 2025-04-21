use crate::expr::{ExprType, LambdaExpr};

pub(crate) fn church_inner(x: i32) -> LambdaExpr {
    if x == 0 {
        LambdaExpr {
            expr_type: ExprType::Var,
            id: 'x' as usize,
            children: vec![],
        }
    } else {
        LambdaExpr {
            expr_type: ExprType::App,
            id: 0,
            children: vec![
                LambdaExpr{
                    expr_type: ExprType::Var,
                    id: 'f' as usize,
                    children: vec![]
                },
                church_inner(x - 1)
            ]
        }
    }
}

pub(crate) fn church(x: i32) -> LambdaExpr {
    LambdaExpr {
        expr_type: ExprType::Abs,
        id: 'f' as usize,
        children: vec![
            LambdaExpr {
                expr_type: ExprType::Abs,
                id: 'x' as usize,
                children: vec![
                    church_inner(x)
                ]
            }
        ]
    }
}

pub(crate) fn unchurch(x: &LambdaExpr) -> i32 {
    match x.expr_type {
        ExprType::Var => 0,
        ExprType::Abs => unchurch(&x.children[0].children[0]),
        ExprType::App => 1 + unchurch(&x.children[1])
    }
}