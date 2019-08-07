
use crate::object::*;
use crate::functions::*;

use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

/// Immutable storage of bindings
#[derive(Debug)]
pub struct Scope {
    map : HashMap<String, Rc<Object>>,
    parent : Option<Rc<RefCell<Scope>>>
}

impl Scope {
    pub fn get(&self, key : &str) -> Option<Rc<Object>> {
        self.map.get(key)
            .map(Rc::clone)
            .or_else(|| self.parent.as_ref()
                .and_then(|p| p.borrow().map.get(key).map(Rc::clone)))
    }
    pub fn bind(&mut self, key: String, value: Rc<Object>) {
        self.map.insert(key, value);
    }
    pub fn new(items: &[(String, Rc<Object>)], parent: Option<Rc<RefCell<Scope>>>) -> Scope {
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
        ("car".to_string(), Rc::new(Object::Function(Function::Pointer(car)))),
        ("cdr".to_string(), Rc::new(Object::Function(Function::Pointer(cdr)))),
        ("cons".to_string(), Rc::new(Object::Function(Function::Pointer(cons)))),
        ("list".to_string(), Rc::new(Object::Function(Function::Pointer(list)))),
        ("length".to_string(), Rc::new(Object::Function(Function::Pointer(length)))),
    ], None);
}