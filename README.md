# Multicall

This library provides the `multicall!` macro, which allows you to apply multiple operations
to one object without writing the name of the object again and again.

## Syntax:
```rs
multicall! {
    expr:
    operation;
    operation;
    operation;
    ...
    {
        subexpr:
        operation;
        operation;
        operation;
        ...
        ...
    }; // this semicolon is mandatory.
}
```

## Evaluates to:
```rs
{
    let __multicall_item__ = expr;
    __multicall_item__.operation;
    __multicall_item__.operation;
    __multicall_item__.operation;
    ...
    {
        let __multicall_item__ = __multicall_item__.subexpr;
        __multicall_item__.operation;
        __multicall_item__.operation;
        __multicall_item__.operation;
        ...
    };
}
```

## Example:
```rs
use multicall::multicall;
use std::ops::AddAssign;
#[derive(Debug)]
struct Test { a: u32, b: i32 }

fn main() {
    let mut test = Test { a: 0, b: 0 };
    multicall! {
        &mut test:
        a = 5;
        b = 6;
        {
            b:
            add_assign(500);
        };
        {
            a:
            add_assign(58);
        };
        a.add_assign(100 - 58);
    }
    println!("{test:?}");
}
```
More in examples/.
