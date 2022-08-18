use super::error::BuilderError;
use super::sqlizer::Sqlizer;
use std::collections::HashMap;
use std::rc::Rc;

enum ValType {
    Any(Rc<dyn Sqlizer>),
    Vec(Vec<Rc<dyn Sqlizer>>),
}

pub struct Builder {
    m: HashMap<String, ValType>,
}

impl Builder {
    pub fn new() -> Self {
        Self { m: HashMap::new() }
    }

    pub fn set(&mut self, k: String, v: Rc<dyn Sqlizer>) -> &mut Self {
        self.m.insert(k, ValType::Any(v));

        self
    }

    pub fn get(&self, k: &str) -> Option<Rc<dyn Sqlizer>> {
        if let Some(v) = self.m.get(k) {
            match v {
                ValType::Any(v) => Some(Rc::clone(v)),
                _ => None,
            }
        } else {
            None
        }
    }

    pub fn get_vec(&self, k: &str) -> Option<Vec<Rc<dyn Sqlizer>>> {
        if let Some(v) = self.m.get(k) {
            match v {
                ValType::Vec(v) => {
                    Some(v.iter().map(|x| Rc::clone(x)).collect())
                }
                _ => None,
            }
        } else {
            None
        }
    }

    pub fn extend(&mut self, k: &str, v: Rc<dyn Sqlizer>) -> Result<(), BuilderError> {
        if let Some(vt) = self.m.get_mut(k) {
            if let ValType::Vec(ref mut ve) = *vt {
                ve.push(v);

                Ok(())
            } else {
                Err(BuilderError::NotVec)
            }
        } else {
            self.m.insert(k.to_owned(), ValType::Vec(vec![v]));

            Ok(())
        }
    }
}
