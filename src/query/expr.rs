use super::sqlizer::Sqlizer;
use std::error::Error;
use std::rc::Rc;

struct SingleExpr<A: 'static> {
    sign: &'static str,
    col: String,
    val: Rc<A>,
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

type Eq<A> = SingleExpr<A>;

pub fn eq<A: 'static>(col: String, val: A) -> Rc<dyn Sqlizer<A>> {
    Rc::new(Eq {
        sign: "=",
        col: col,
        val: Rc::new(val),
    })
}
