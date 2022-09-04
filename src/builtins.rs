use self::types::BuiltinTypes;
use crate::{
    get_native_prop,
    value::{ArgPattern, Value},
    Interpreter,
};

mod types;

/// A struct containing all builtins types and values.
pub struct Builtins {
    pub types: BuiltinTypes,
    pub print: Value,
}

macro_rules! impl_builtin_names {
    ($self:ident, $name:ident, [$($names:ident),*]) => {
        match $name {
            $(stringify!($names) => Some($self.$names.clone()),)*
            _ => None,
        }
    };
}

impl Builtins {
    /// Create new instance of builtins.
    pub fn new() -> Self {
        Self {
            types: BuiltinTypes::new(),
            print: Value::Native(print, ArgPattern::Any),
        }
    }

    /// Get a builtin by name.
    /// Returns `None` if `name` does not exist.
    pub fn resolve(&self, name: &str) -> Option<Value> {
        impl_builtin_names!(self, name, [print])
    }
}

fn print(interp: &mut Interpreter, args: &[Value]) -> Value {
    for (i, arg) in args.iter().enumerate() {
        let display = get_native_prop!(interp, arg, display);
        if let Value::String(str) = interp.call_with_return(display, &[arg.clone()]) {
            if i == 0 {
                print!("{}", str.value());
            } else {
                print!(", {}", str.value());
            }
        } else {
            panic!("expected `$display` to return str");
        }
    }

    println!();

    Value::Null
}


