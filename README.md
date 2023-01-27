# Multicall

This library provides the `multicall!` macro, which allows you to apply multiple operations
to one object without writing the name of the object again and again.

## Syntax:
```rs
let mut test_variable = 1;
multicall! {
    expr:
    operation;
    set test_variable = operation;
    exec normal_operation(#);
    operation;
    ...
    {
        subexpr:
        operation;
        set test_variable += operation;
        exec normal_operation(#);
        operation;
        ...
    }; // this semicolon is mandatory.
}
```

## Evaluates to:
```rs
let mut test_variable = 1;
{
    let __multicall_item__ = expr;
    __multicall_item__.operation;
    test_variable = __multicall_item__.operation;
    normal_operation(__multicall_item__);
    __multicall_item__.operation;
    ...
    {
        let __multicall_item__ = __multicall_item__.subexpr;
        __multicall_item__.operation;
        test_variable += __multicall_item__.operation;
        normal_operation(__multicall_item__);
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
    let b_plus_five;
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
        set b_plus_five = b + 5;
        exec println!("{}, {}", #.a, #.b);
    }
    println!("{test:?}");
}
```
More in examples/.
