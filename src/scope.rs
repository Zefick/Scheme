use crate::functions::*;
use crate::lists::*;
use crate::logic::*;
use crate::math::*;
use crate::object::*;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug)]
pub struct Scope {
    map: RefCell<HashMap<String, Rc<Object>>>,
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
        let mut scope = HashMap::new();
        for item in items {
            scope.insert(item.0.clone(), Rc::clone(&item.1));
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
        ("#t".to_string(), Rc::new(Object::Boolean(true))),
        ("#f".to_string(), Rc::new(Object::Boolean(false))),
        ("cons".to_string(), Rc::new(Object::Function(Function::Pointer(cons)))),
        ("list".to_string(), Rc::new(Object::Function(Function::Pointer(list)))),
        ("length".to_string(), Rc::new(Object::Function(Function::Pointer(length)))),
        ("map".to_string(), Rc::new(Object::Function(Function::Pointer(fn_map)))),
        ("boolean?".to_string(), Rc::new(Object::Function(Function::Pointer(is_boolean)))),
        ("list?".to_string(), Rc::new(Object::Function(Function::Pointer(is_list)))),
        ("pair?".to_string(), Rc::new(Object::Function(Function::Pointer(is_pair)))),
        ("null?".to_string(), Rc::new(Object::Function(Function::Pointer(is_null)))),
        ("not".to_string(), Rc::new(Object::Function(Function::Pointer(logic_not)))),
        ("eq?".to_string(), Rc::new(Object::Function(Function::Pointer(fn_eq)))),
        ("eqv?".to_string(), Rc::new(Object::Function(Function::Pointer(fn_eqv)))),
        ("equal?".to_string(), Rc::new(Object::Function(Function::Pointer(fn_equal)))),
        ("number?".to_string(), Rc::new(Object::Function(Function::Pointer(is_number)))),
        ("integer?".to_string(), Rc::new(Object::Function(Function::Pointer(is_integer)))),
        ("real?".to_string(), Rc::new(Object::Function(Function::Pointer(is_real)))),
        ("=".to_string(), Rc::new(Object::Function(Function::Pointer(num_eqv)))),
        ("<".to_string(), Rc::new(Object::Function(Function::Pointer(num_less)))),
        (">".to_string(), Rc::new(Object::Function(Function::Pointer(num_greater)))),
        ("+".to_string(), Rc::new(Object::Function(Function::Pointer(num_plus)))),
        ("-".to_string(), Rc::new(Object::Function(Function::Pointer(num_minus)))),
        ("*".to_string(), Rc::new(Object::Function(Function::Pointer(num_mul)))),
        ("/".to_string(), Rc::new(Object::Function(Function::Pointer(num_div)))),
    ];
    Scope{
        map: RefCell::new(HashMap::from(bindings)),
        parent: None,
    }
}
