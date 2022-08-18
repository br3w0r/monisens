use std::any::Any;
use std::error::Error;
use std::rc::Rc;

pub trait Sqlizer<A: 'static> {
    fn sql(&self) -> Result<(String, Option<Vec<Rc<A>>>), Box<dyn Error>>;
}

// Implement the trait for all references that already implement the trait
impl<'a, A: 'static, T: ?Sized + Sqlizer<A>> Sqlizer<A> for &'_ T {
    fn sql(&self) -> Result<(String, Option<Vec<Rc<A>>>), Box<dyn Error>> {
        <T as Sqlizer<A>>::sql(self)
    }
}

pub enum PredType<A> {
    String(String),
    Sql(Rc<dyn Sqlizer<A>>),
}

pub struct Part<A> {
    pred: PredType<A>,
    args: Option<Vec<Rc<A>>>,
}

impl<A> Part<A> {
    pub fn new(pred: PredType<A>, args: Option<Vec<Rc<A>>>) -> Self {
        Self {
            pred: pred,
            args: args,
        }
    }
}

impl<A: 'static> Sqlizer<A> for Part<A> {
    fn sql(&self) -> Result<(String, Option<Vec<Rc<A>>>), Box<dyn Error>> {
        match &self.pred {
            PredType::String(ref s) => Ok((
                s.to_owned(),
                match &self.args {
                    Some(v) => Some(v.iter().map(|x| Rc::clone(x)).collect()),
                    None => None,
                },
            )),
            PredType::Sql(s) => s.sql(),
        }
    }
}
