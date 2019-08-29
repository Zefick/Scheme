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
    map: RefCell<HashMap<String, Rc<Object>>>,
    parent: Option<Rc<Scope>>,
}

impl Scope {
    pub fn get(&self, key: &str) -> Option<Rc<Object>> {
        (self.map.borrow().get(key).map(Rc::clone))
            .or_else(|| self.parent.as_ref().and_then(|p| p.get(key)))
    }
    pub fn bind(&self, key: &String, value: Rc<Object>) {
        self.map.borrow_mut().insert(key.clone(), value);
    }
    pub fn new(items: &[(String, Rc<Object>)], parent: Option<&Rc<Scope>>) -> Rc<Scope> {
        let mut scope = HashMap::new();
        for item in items {
            scope.insert(item.0.clone(), Rc::clone(&item.1));
        }
        return Rc::new(Scope {
            map: RefCell::new(scope),
            parent: parent.map(Rc::clone),
        });
    }
}

/// Global bindings storage accessible from everywhere.
/// Contains core functions and constants like `#t` and `#f`
#[rustfmt::skip]
pub fn get_global_scope() -> Rc<Scope> {
    return Scope::new(&[
        ("#t".to_string(), Rc::new(Object::Boolean(true))),
        ("#f".to_string(), Rc::new(Object::Boolean(false))),
        ("cons".to_string(), Rc::new(Object::Function(Function::Pointer(cons)))),
        ("list".to_string(), Rc::new(Object::Function(Function::Pointer(list)))),
        ("length".to_string(), Rc::new(Object::Function(Function::Pointer(length)))),
        ("apply".to_string(), Rc::new(Object::Function(Function::Pointer(fn_apply)))),
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
    ], None);
}
