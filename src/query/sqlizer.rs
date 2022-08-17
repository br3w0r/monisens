use std::any::Any;
use std::error::Error;
use std::rc::Rc;

pub trait Sqlizer<'a> {
    fn sql(&self) -> Result<(String, Option<Vec<Rc<dyn Any>>>), Box<dyn Error>>;
}

// Implement the trait for all references that already implement the trait
impl<'a, T: ?Sized + Sqlizer<'a>> Sqlizer<'a> for &'_ T {
    fn sql(&self) -> Result<(String, Option<Vec<Rc<dyn Any>>>), Box<dyn Error>> {
        <T as Sqlizer>::sql(self)
    }
}

pub enum PredType<'a> {
    String(String),
    Sql(&'a dyn Sqlizer<'a>),
}

pub struct Part<'a> {
    pred: PredType<'a>,
    args: Option<Vec<Rc<dyn Any>>>,
}

impl<'a> Part<'a> {
    pub fn new(pred: PredType<'a>, args: Option<Vec<Rc<dyn Any>>>) -> Self {
        Self {
            pred: pred,
            args: args,
        }
    }
}

impl<'a> Sqlizer<'a> for Part<'a> {
    fn sql(&self) -> Result<(String, Option<Vec<Rc<dyn Any>>>), Box<dyn Error>> {
        match self.pred {
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
