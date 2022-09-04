use std::{
    env,
    fs::read_to_string,
    io::{stdin, Read},
};

use san_script::{
    builtins::Builtins,
    frame::Frame,
    parser::Parser,
    ptr::{Ptr, PtrMut},
    CodeBuilder, Interpreter,
};

fn main() {
    let mut content = String::new();

    if let Some(path) = env::args().nth(1) {
        content = read_to_string(path).expect("error while reading source file");
    } else {
        stdin()
            .read_to_string(&mut content)
            .expect("error while reading stdin");
    }

    let mut code_builder = CodeBuilder::new(0);

    let parser = Parser::new();
    let module = parser.parse("test", &content).unwrap();

    code_builder.compile_module(&module);

    let code = Ptr::new(code_builder.build());
    let frame = PtrMut::new(Frame::new(code, None, None));

    let mut interpreter = Interpreter::new(frame, Builtins::new());
    interpreter.run();
}
