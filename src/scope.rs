use crate::functions::*;
use crate::lists::*;
use crate::logic::*;
use crate::math::*;
use crate::object::*;

use ahash::RandomState;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug)]
pub struct Scope {
    map: RefCell<HashMap<String, Rc<Object>, RandomState>>,
    parent: Option<Rc<Scope>>,
}

impl Scope {
    pub fn get(&self, key: &str) -> Option<Rc<Object>> {
        (self.map.borrow().get(key).map(Rc::clone))
            .or_else(|| self.parent.as_ref().and_then(|p| p.get(key)))
    }
    pub fn bind(&self, key: &str, value: Rc<Object>) {
        self.map.borrow_mut().insert(key.to_string(), value);
    }
    pub fn new(items: &[(String, Rc<Object>)], parent: &Rc<Scope>) -> Self {
        let mut scope = HashMap::with_capacity_and_hasher(items.len(), RandomState::new());
        for item in items {
            scope.insert(item.0.clone(), Rc::clone(&item.1));
        }
        Scope { map: RefCell::new(scope), parent: Some(parent.clone()) }
    }
    pub fn new_owned(items: Vec<(String, Rc<Object>)>, parent: &Rc<Scope>) -> Self {
        let mut scope = HashMap::with_capacity_and_hasher(items.len(), RandomState::new());
        for (key, value) in items {
            scope.insert(key, value);
        }
        Scope { map: RefCell::new(scope), parent: Some(parent.clone()) }
    }
    pub fn from_global() -> Self {
        Self::new(&[], &Rc::new(get_global_scope()))
    }
    pub fn from_scope(scope: &Rc<Scope>) -> Self {
        Self::new(&[], scope)
    }
}

/// Global bindings storage accessible from everywhere.
/// Contains core functions and constants like `#t` and `#f`
#[rustfmt::skip]
fn get_global_scope() -> Scope {
    let bindings = [
        ("#t", Object::Boolean(true)),
        ("#f", Object::Boolean(false)),
        ("cons", Function::from_pointer(cons)),
        ("list", Function::from_pointer(list)),
        ("length", Function::from_pointer(length)),
        ("map", Function::from_pointer(fn_map)),
        ("boolean?", Function::from_pointer(is_boolean)),
        ("list?", Function::from_pointer(is_list)),
        ("pair?", Function::from_pointer(is_pair)),
        ("null?", Function::from_pointer(is_null)),
        ("not", Function::from_pointer(logic_not)),
        ("eq?", Function::from_pointer(fn_eq)),
        ("eqv?", Function::from_pointer(fn_eqv)),
        ("equal?", Function::from_pointer(fn_equal)),
        ("number?", Function::from_pointer(is_number)),
        ("integer?", Function::from_pointer(is_integer)),
        ("real?", Function::from_pointer(is_real)),
        ("=", Function::from_pointer(num_eqv)),
        ("<", Function::from_pointer(num_less)),
        (">", Function::from_pointer(num_greater)),
        ("+", Function::from_pointer(num_plus)),
        ("-", Function::from_pointer(num_minus)),
        ("*", Function::from_pointer(num_mul)),
        ("/", Function::from_pointer(num_div)),
        ("quotient", Function::from_pointer(quotient)),
        ("remainder", Function::from_pointer(remainder)),
        ("modulo", Function::from_pointer(modulo)),

    ];
    let mut map = HashMap::with_capacity_and_hasher(bindings.len(), RandomState::new());
    for (s, obj) in bindings {
        map.insert(s.to_string(), Rc::new(obj));
    }
    Scope { map: RefCell::new(map), parent: None }
}
