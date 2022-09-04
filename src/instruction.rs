pub type ConstantIndex = usize;
pub type NameIndex = usize;

#[derive(Debug, Clone, Copy)]
pub enum Instruction {
    Call(usize),
    JumpFalse(usize),
    Jump(usize),
    Return,
    CreateFunction,
    CreateList(usize),
    Pop,
    LoadConstant(ConstantIndex),
    Declare(NameIndex),
    LoadVariable(NameIndex),
    StoreVariable(NameIndex),
    StoreProperty(NameIndex),
    LoadProperty(NameIndex),
    Exit(usize),
    StoreSubscript,
    LoadSubscript,
    Add,
    Subtract,
    Multiply,
    Divide,
    Equals,
    NotEquals,
    LessThan,
    GreaterThan,
    LessThanOrEqual,
    GreaterThanOrEqual
}
