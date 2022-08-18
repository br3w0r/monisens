use super::sqlizer::Sqlizer;
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

macro_rules! single_expr {
    ($name:ident, $sign:literal) => {
        pub fn $name<A: 'static>(col: String, val: A) -> Rc<dyn Sqlizer<A>> {
            SingleExpr::new($sign, col, val)
        }
    };
}

single_expr!(eq, "=");
single_expr!(gt, ">");
single_expr!(gte, ">=");
single_expr!(lt, "<");
single_expr!(lte, "<=");
