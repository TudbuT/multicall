use multicall::multicall;
use std::ops::AddAssign;
#[derive(Debug)]
struct Test {
    a: u32,
    b: i32,
}

impl Test {
    fn print_a(&self) {
        println!("{}", self.a);
    }
    fn print_b(&self) {
        println!("{}", self.b);
    }
}

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
    multicall! {
        &test:
        print_a();
        print_b();
    }
    println!("{test:?}");
}
