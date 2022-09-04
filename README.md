<h1 align="center">San-Script </h1>
 
 <h3 align="center">
An experimental hobby language! ⚡
</h3>
<br> 

## Inspiration

San-script is a language that takes some syntax idea's from Rust, Python and Javascript. It compiles to bytecode that looks very similliar to Python bytecode. 

## Usage 

```bash
cargo build --release
./target/release/san-script <san-script-source> 
```

## Syntax

Variable declaration:
```
let x = 10;
```

Assignment:
```
x = "foo";
```

Functions are expressions:
```
let func = fn(){ 
  print("hello world!");
};
```
Functions without a body return the result of the expression;
```
let add_one = fn(x) x + 1;
```
Functions with a body return a result with `^`:
```
let is_zero = fn(x) {
  ^ x == 0;
};
```
Closures work!
```
let print_foo_factory = fn(){
  let x = "foo";
  ^ fn() print(x);
};
let print_foo = print_foo_factory();
print_foo();
>>> foo
```


