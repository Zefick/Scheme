
use crate::object::*;
use crate::eval::*;

use std::collections::HashMap;
use std::rc::Rc;

/// Immutable storage of bindings
pub struct Scope {
    map : HashMap<String, Rc<Object>>,
    parent : Option<Rc<Scope>>
}

impl Scope {
    pub fn get(&self, key : &str) -> Option<&Rc<Object>> {
        self.map.get(key).or_else(||
            self.parent.as_ref().and_then(|p| p.get(key)))
    }
    pub fn new (items: &[(&str, Rc<Object>)], parent: Option<Rc<Scope>>) -> Scope {
        let mut scope = HashMap::<String, Rc<Object>>::new();
        for item in items {
           scope.insert(item.0.to_string(), Rc::clone(&item.1));
        };
        return Scope{map: scope, parent};
    }
}

/// Global bindings storage accessible from everywhere.
/// Contains core functions and constants like `#t` and `#f`
pub fn get_global_scope() -> Scope {
    return Scope::new(&[
        ("#t", Rc::new(Object::Boolean(true))),
        ("#f", Rc::new(Object::Boolean(false))),
        ("car", Rc::new(Object::Function(car))),
        ("cdr", Rc::new(Object::Function(cdr))),
        ("length", Rc::new(Object::Function(length)))
    ], None);
}