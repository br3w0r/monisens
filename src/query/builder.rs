use super::error::BuilderError;
use super::sqlizer::Sqlizer;
use std::collections::HashMap;
use std::rc::Rc;

enum ValType<'a> {
    Any(Rc<dyn 'a + Sqlizer<'a>>),
    Vec(Vec<Rc<dyn 'a + Sqlizer<'a>>>),
}

pub struct Builder<'a> {
    m: HashMap<String, ValType<'a>>,
}

impl<'a> Builder<'a> {
    pub fn new() -> Self {
        Self { m: HashMap::new() }
    }

    pub fn set<T: 'a + Sqlizer<'a>>(&mut self, k: String, v: T) -> &mut Self {
        self.m.insert(k, ValType::Any(Rc::new(v)));

        self
    }

    pub fn get(&self, k: &str) -> Option<Rc<dyn 'a + Sqlizer<'a>>> {
        if let Some(v) = self.m.get(k) {
            match v {
                ValType::Any(v) => Some(Rc::clone(v)),
                _ => None,
            }
        } else {
            None
        }
    }

    pub fn get_vec(&self, k: &str) -> Option<Vec<Rc<dyn 'a + Sqlizer<'a>>>> {
        if let Some(v) = self.m.get(k) {
            match v {
                ValType::Vec(v) => Some(v.iter().map(|x| Rc::clone(x)).collect()),
                _ => None,
            }
        } else {
            None
        }
    }

    pub fn extend<T: 'a + Sqlizer<'a>>(&mut self, k: &str, v: T) -> Result<(), BuilderError> {
        if let Some(vt) = self.m.get_mut(k) {
            if let ValType::Vec(ref mut ve) = *vt {
                ve.push(Rc::new(v));

                Ok(())
            } else {
                Err(BuilderError::NotVec)
            }
        } else {
            let mut ve: Vec<Rc<dyn 'a + Sqlizer<'a>>> = Vec::new();
            ve.push(Rc::new(v));
            self.m.insert(k.to_owned(), ValType::Vec(ve));

            Ok(())
        }
    }
}
