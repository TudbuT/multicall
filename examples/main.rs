use multicall::multicall;

#[derive(Debug)]
struct Test {
    a: u32,
    b: i32,
    c: u8,
    greeting: String,
    to_greet: String,
}

impl Test {
    fn print_a(&self) {
        println!("{}", self.a);
    }
    fn print_b(&self) {
        println!("{}", self.b);
    }
    fn set_greeting(&mut self, greeting: &str) {
        self.greeting = greeting.to_owned();
    }
    fn set_to_greet(&mut self, to_greet: &str) {
        self.to_greet = to_greet.to_owned();
    }

    fn to_string(&self) -> String {
        multicall! {
            self:
            exec return format!(
                "{}, {}! Have some stuff: {}, {}, {}",
                #.greeting,
                #.to_greet,
                #.a,
                #.b,
                #.c,
            );
        }
    }
}

fn main() {
    let mut test = Test { a: 0, b: 0, c: 0, greeting: "Hello".to_owned(), to_greet: "Nobody".to_owned() };
    let mut value = 5;
    multicall! {
        &mut test:
        a = value;
        b = (#.a + value) as i32;
        print_a();
        print_b();
        exec println!("{}", #.to_string());
        c = 48;
        set_greeting("Hello");
        set_to_greet("multicall");
        exec println!("{}", #.to_string());
        set value = c as u32;
    }
    println!("{}", value);
}
