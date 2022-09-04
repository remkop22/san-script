
#[derive(Debug)]
pub struct Module {
    pub name: String,
    pub body: Vec<Statement>,
}

#[derive(Debug)]
pub enum AssignmentTarget {
    Identifier(String),
    Property(Expression, String),
    Subscript(Expression, Expression),
}

#[derive(Debug)]
pub enum Statement {
    Expression(Expression),
    Return(Expression),
    Assignment {
        target: AssignmentTarget,
        source: Expression,
    },
    Declaration {
        ident: String,
        assign: Option<Expression>,
    },
    If {
        cond: Expression,
        body: Vec<Statement>,
        else_body: Vec<Statement>
    }
}

#[derive(Debug)]
pub enum Operator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Equals,
    NotEquals,
    LessThan,
    GreaterThan,
    LessThanOrEqual,
    GreaterThanOrEqual,
}

#[derive(Debug)]
pub enum Expression {
    Identifier(String),
    Integer(i64),
    Float(f64),
    String(String),
    List(Vec<Expression>),
    Object(Vec<(String, Expression)>),
    Property(Box<Expression>, String),
    Subscript(Box<Expression>, Box<Expression>),
    Function {
        params: Vec<String>,
        body: Vec<Statement>,
    },
    FunctionCall {
        target: Box<Expression>,
        args: Vec<Expression>,
    },
    Operation {
        lhs: Box<Expression>,
        op: Operator,
        rhs: Box<Expression>,
    },
}
