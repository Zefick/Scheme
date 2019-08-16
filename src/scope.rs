use crate::functions::*;
use crate::lists::*;
use crate::logic::*;
use crate::math::*;
use crate::object::*;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Immutable storage of bindings
#[derive(Debug)]
pub struct Scope {
    map: HashMap<String, Rc<Object>>,
    parent: Option<Rc<RefCell<Scope>>>,
}

impl Scope {
    pub fn get(&self, key: &str) -> Option<Rc<Object>> {
        self.map
            .get(key)
            .map(Rc::clone)
            .or_else(|| self.parent.as_ref().and_then(|p| p.borrow().get(key)))
    }
    pub fn bind(&mut self, key: String, value: Rc<Object>) {
        self.map.insert(key, value);
    }
    pub fn new(items: &[(String, Rc<Object>)], parent: Option<&Rc<RefCell<Scope>>>) -> Scope {
        let mut scope = HashMap::new();
        for item in items {
            scope.insert(item.0.clone(), Rc::clone(&item.1));
        }
        return Scope {
            map: scope,
            parent: parent.map(Rc::clone),
        };
    }
}

/// Global bindings storage accessible from everywhere.
/// Contains core functions and constants like `#t` and `#f`
#[rustfmt::skip]
pub fn get_global_scope() -> Scope {
    return Scope::new(&[
        ("#t".to_string(), Rc::new(Object::Boolean(true))),
        ("#f".to_string(), Rc::new(Object::Boolean(false))),
        ("car".to_string(), Rc::new(Object::Function(Function::Pointer(car)))),
        ("cdr".to_string(), Rc::new(Object::Function(Function::Pointer(cdr)))),
        ("cons".to_string(), Rc::new(Object::Function(Function::Pointer(cons)))),
        ("list".to_string(), Rc::new(Object::Function(Function::Pointer(list)))),
        ("length".to_string(), Rc::new(Object::Function(Function::Pointer(length)))),

        ("boolean?".to_string(), Rc::new(Object::Function(Function::Pointer(is_boolean)))),
        ("list?".to_string(), Rc::new(Object::Function(Function::Pointer(is_list)))),
        ("pair?".to_string(), Rc::new(Object::Function(Function::Pointer(is_pair)))),
        ("not".to_string(), Rc::new(Object::Function(Function::Pointer(logic_not)))),

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
    ], None);
}
