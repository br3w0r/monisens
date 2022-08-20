use super::error::ValuesError;
use super::tool;
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

// Common sql part used primarily for string types such as column names
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

// Sqlizer for INSERT's VALUES statement
pub struct Values<A>(Vec<Rc<A>>);

impl<A: 'static> Sqlizer<A> for Values<A> {
    fn sql(&self) -> Result<(String, Option<Vec<Rc<A>>>), Box<dyn Error>> {
        if self.0.len() == 0 {
            return Err(ValuesError::NoValues.into());
        }

        let placeholders = tool::placeholders(self.0.len());
        let mut sql = String::with_capacity(placeholders.len() + 2);

        sql.push('(');
        sql.push_str(&placeholders);
        sql.push(')');

        Ok((sql, Some(self.0.iter().map(|x| Rc::clone(x)).collect())))
    }
}

impl<A: 'static> From<Vec<A>> for Values<A> {
    fn from(v: Vec<A>) -> Self {
        let mut res = Vec::with_capacity(v.len());
        for x in v {
            res.push(Rc::new(x));
        }

        Values(res)
    }
}
