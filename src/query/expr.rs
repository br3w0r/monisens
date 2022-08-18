use super::sqlizer::Sqlizer;
use std::any::Any;
use std::error::Error;
use std::rc::Rc;

struct Eq<A: 'static> {
    col: String,
    val: Rc<A>,
}

impl<A: 'static> Sqlizer<A> for Eq<A> {
    fn sql(&self) -> Result<(String, Option<Vec<Rc<A>>>), Box<dyn Error>> {
        let mut s = String::with_capacity(self.col.len() + 4);
        s.push_str(&self.col);
        s.push_str(" = ?");

        Ok((s, Some(vec![Rc::clone(&self.val)])))
    }
}

pub fn eq<A: 'static>(col: String, val: A) -> Rc<dyn Sqlizer<A>> {
    Rc::new(Eq {
        col: col,
        val: Rc::new(val),
    })
}
