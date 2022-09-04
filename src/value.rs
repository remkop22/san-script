use crate::builtins::Builtins;
use crate::frame::{Code, Frame, Function};
use crate::interpreter::Interpreter;
use crate::ptr::{Ptr, PtrMut};
use std::collections::HashMap;

pub struct Type {
    pub name: Ptr<String>,
    pub base: Option<Ptr<Type>>,
    pub call: Option<Value>,
    pub add: Option<Value>,
    pub subtract: Option<Value>,
    pub multiply: Option<Value>,
    pub divide: Option<Value>,
    pub equals: Option<Value>,
    pub not_equals: Option<Value>,
    pub less_than: Option<Value>,
    pub greater_than: Option<Value>,
    pub less_than_or_equal: Option<Value>,
    pub greater_than_or_equal: Option<Value>,
    pub display: Option<Value>,
    pub get_property: Option<Value>,
    pub set_property: Option<Value>,
    pub get_subscript: Option<Value>,
    pub set_subscript: Option<Value>,
    pub properties: HashMap<Ptr<String>, Value>,
}

impl Type {
    pub fn new(name: Ptr<String>, base: Ptr<Type>) -> Self {
        Self::empty(name, Some(base))
    }

    fn empty(name: Ptr<String>, base: Option<Ptr<Type>>) -> Self {
        Self {
            name,
            base,
            call: None,
            add: None,
            subtract: None,
            multiply: None,
            divide: None,
            display: None,
            get_property: None,
            set_property: None,
            get_subscript: None,
            set_subscript: None,
            less_than_or_equal: None,
            greater_than_or_equal: None,
            greater_than: None,
            less_than: None,
            equals: None,
            not_equals: None,
            properties: HashMap::new(),
        }
    }

    pub fn root(name: Ptr<String>) -> Self {
        Self::empty(name, None)
    }
}

pub struct Object {
    ty: Ptr<Type>,
    properties: HashMap<Ptr<String>, Value>,
}

impl Object {
    pub fn new(ty: Ptr<Type>, properties: HashMap<Ptr<String>, Value>) -> Self {
        Self { ty, properties }
    }

    pub fn get_property(&self, name: &Ptr<String>) -> Option<Value> {
        self.properties.get(name).cloned()
    }

    pub fn set_property(&mut self, name: Ptr<String>, value: Value) {
        self.properties.insert(name, value);
    }
}

pub type NativeFunction = fn(&mut Interpreter, &[Value]) -> Value;

#[derive(Clone)]
pub enum ArgPattern {
    Any,
    Exact(usize),
    Min(usize),
    Max(usize),
    Range(usize, usize),
}

#[derive(Clone)]
pub enum Value {
    Null,
    Object(PtrMut<Object>),
    List(PtrMut<Vec<Value>>),
    String(Ptr<String>),
    Integer(i64),
    Float(f64),
    Bool(bool),
    Code(Ptr<Code>),
    Frame(PtrMut<Frame>),
    Function(Ptr<Function>),
    Bound(Ptr<Value>, Ptr<Value>),
    Native(NativeFunction, ArgPattern),
    Type(Ptr<Type>),
}

impl Value {
    pub fn ty(&self, builtins: &Builtins) -> Ptr<Type> {
        match self {
            Self::Object(obj) => obj.value().ty.clone(),
            Self::Bound(obj, _) => obj.value().ty(builtins),
            Self::List(_) => builtins.types.list.clone(),
            Self::String(_) => builtins.types.string.clone(),
            Self::Float(_) => builtins.types.float.clone(),
            Self::Integer(_) => builtins.types.integer.clone(),
            Self::Bool(_) => builtins.types.bool.clone(),
            Self::Function(_) => builtins.types.function.clone(),
            Self::Frame(_) => builtins.types.frame.clone(),
            Self::Native(..) => builtins.types.native.clone(),
            Self::Code(_) => builtins.types.code.clone(),
            Self::Null => builtins.types.null.clone(),
            Self::Type(_) => builtins.types.ty.clone(),
        }
    }

    pub fn as_bool(&self, interp: &Interpreter) -> bool {
        match self {
            Self::Integer(v) => *v != 0,
            Self::Float(v) => *v != 0.0,
            Self::Bool(b) => *b,
            _ => panic!(
                "expected bool, but found {}",
                self.ty(interp.builtins()).value().name.value()
            ),
        }
    }

    pub fn list(&self, interp: &Interpreter) -> PtrMut<Vec<Value>> {
        match self {
            Self::List(v) => v.clone(),
            _ => panic!(
                "expected list, but found {}",
                self.ty(interp.builtins()).value().name.value()
            ),
        }
    }

    pub fn string(&self, interp: &Interpreter) -> Ptr<String> {
        match self {
            Self::String(v) => v.clone(),
            _ => panic!(
                "expected string, but found {}",
                self.ty(interp.builtins()).value().name.value()
            ),
        }
    }

    pub fn bool(&self, interp: &Interpreter) -> bool {
        match self {
            Self::Bool(v) => *v,
            _ => panic!(
                "expected bool, but found {}",
                self.ty(interp.builtins()).value().name.value()
            ),
        }
    }

    pub fn int(&self, interp: &Interpreter) -> i64 {
        match self {
            Self::Integer(v) => *v,
            _ => panic!(
                "expected int, but found {}",
                self.ty(interp.builtins()).value().name.value()
            ),
        }
    }

    pub fn as_float(&self, interp: &Interpreter) -> f64 {
        match self {
            Value::Integer(v) => *v as f64,
            Value::Float(v) => *v,
            _ => panic!(
                "expected number, but found {}",
                self.ty(interp.builtins()).value().name.value()
            ),
        }
    }

    pub fn float(&self, interp: &Interpreter) -> f64 {
        match self {
            Value::Float(v) => *v,
            _ => panic!(
                "expected int, but found {}",
                self.ty(interp.builtins()).value().name.value()
            ),
        }
    }

    pub fn as_int(&self, interp: &Interpreter) -> i64 {
        match self {
            Value::Integer(v) => *v,
            Value::Float(v) => *v as i64,
            _ => panic!(
                "expected number argument but found {}",
                self.ty(interp.builtins()).value().name.value()
            ),
        }
    }
}
