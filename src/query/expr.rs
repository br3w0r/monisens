use super::error::ExprError;
use super::sqlizer::Sqlizer;
use super::tool;

use std::error::Error;
use std::rc::Rc;

struct SingleExpr<A: 'static> {
    sign: &'static str,
    col: String,
    val: Rc<A>,
}

impl<A: 'static> SingleExpr<A> {
    pub fn new(sign: &'static str, col: String, val: A) -> Rc<dyn Sqlizer<A>> {
        Rc::new(SingleExpr {
            sign: sign,
            col: col,
            val: Rc::new(val),
        })
    }
}

impl<A: 'static> Sqlizer<A> for SingleExpr<A> {
    fn sql(&self) -> Result<(String, Option<Vec<Rc<A>>>), Box<dyn Error>> {
        let mut s = String::with_capacity(self.col.len() + 4);
        s.push_str(&self.col);
        s.push_str(" ");
        s.push_str(self.sign);
        s.push_str(" ?");

        Ok((s, Some(vec![Rc::clone(&self.val)])))
    }
}

macro_rules! single_arg_expr {
    ($name:ident, $sign:literal) => {
        pub fn $name<A: 'static>(col: String, val: A) -> Rc<dyn Sqlizer<A>> {
            SingleExpr::new($sign, col, val)
        }
    };
}

single_arg_expr!(eq, "=");
single_arg_expr!(neq, "<>");
single_arg_expr!(gt, ">");
single_arg_expr!(gte, ">=");
single_arg_expr!(lt, "<");
single_arg_expr!(lte, "<=");

struct InExpr<A: 'static> {
    col: String,
    values: Vec<Rc<A>>,
}

impl<A: 'static> Sqlizer<A> for InExpr<A> {
    fn sql(&self) -> Result<(String, Option<Vec<Rc<A>>>), Box<dyn Error>> {
        if self.values.len() == 0 {
            return Err(Box::new(ExprError::NoArgs));
        }

        let sql = self.col.clone() + " IN (" + &tool::placeholders(self.values.len()) + ")";

        let args = {
            if self.values.len() > 0 {
                Some(self.values.iter().map(|v| v.clone()).collect())
            } else {
                None
            }
        };

        Ok((sql, args))
    }
}

pub fn inq<A: 'static>(col: String, mut val: Vec<A>) -> Rc<dyn Sqlizer<A>> {
    Rc::new(InExpr::<A> {
        col: col,
        values: val.drain(..).map(|v| Rc::new(v)).collect(),
    })
}
