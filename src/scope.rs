
use crate::object::*;
use crate::eval::*;

use std::collections::HashMap;
use std::rc::Rc;

/// Immutable storage of bindings
#[derive(Debug)]
pub struct Scope {
    map : HashMap<String, Rc<Object>>,
    parent : Option<Rc<Scope>>
}

impl Scope {
    pub fn get(&self, key : &str) -> Option<&Rc<Object>> {
        self.map.get(key).or_else(||
            self.parent.as_ref().and_then(|p| p.get(key)))
    }
    pub fn new(items: &[(String, Rc<Object>)], parent: Option<Rc<Scope>>) -> Scope {
        let mut scope = HashMap::new();
        for item in items {
           scope.insert(item.0.clone(), Rc::clone(&item.1));
        };
        return Scope{map: scope, parent};
    }
}

/// Global bindings storage accessible from everywhere.
/// Contains core functions and constants like `#t` and `#f`
pub fn get_global_scope() -> Scope {
    return Scope::new(&[
        ("#t".to_string(), Rc::new(Object::Boolean(true))),
        ("#f".to_string(), Rc::new(Object::Boolean(false))),
        ("car".to_string(), Rc::new(Object::Function(car))),
        ("cdr".to_string(), Rc::new(Object::Function(cdr))),
        ("length".to_string(), Rc::new(Object::Function(length))),
        ("begin".to_string(), Rc::new(Object::Function(fn_begin))),
    ], None);
}