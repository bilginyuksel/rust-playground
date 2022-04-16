use std::env::{args, Args};

fn main() {
    let mut args: Args = args();

    let first = args.nth(1).unwrap();
    let operator: char = args.nth(0).unwrap().parse().unwrap();
    let second = args.nth(0).unwrap();

    let first_number = first.parse::<f32>().unwrap();
    let second_number = second.parse::<f32>().unwrap();

    println!("{} {} {} = {}", first, operator, second, calculate(operator, first_number, second_number));
}

fn calculate(operator: char, a: f32, b: f32) -> f32 {
    match operator {
        '+' => a + b,
        '-' => a - b,
        '/' => a / b,
        '*' | 'x' | 'X' => a * b,
        _ => panic!("Invalid operator"),
    }
}
