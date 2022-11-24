use std::io;

use rand::Rng;

fn main() {
    let roll_amount = get_input();

    death_roll(roll_amount)
}

pub fn get_input() -> i32 {
    loop {
        let mut input = String::new();

        println!("enter roll amount between 100 and 10,000 to begin:");
        io::stdin().read_line(&mut input).unwrap();

        let input: i32 = match input.trim().parse::<i32>() {
            Ok(parsed_input) => match parsed_input {
                100..=10000 => parsed_input,
                _ => continue,
            },
            Err(_) => continue,
        };

        return input;
    }
}

pub fn death_roll(roll_amount: i32) {
    println!("player 1: ");
    let mut p1 = String::new();
    io::stdin().read_line(&mut p1).unwrap();

    let mut die1 = roll(roll_amount);

    loop {
        println!("{:?}", die1);

        if die1 == 1 {
            println!("player 1: You Died!");
            break;
        } else {
            println!("player 2: ");
            let mut p1 = String::new();
            io::stdin().read_line(&mut p1).unwrap();
            let die2 = roll(die1);
            println!("{:?}", die2);

            if die2 == 1 {
                println!("player 2: You Died!");
                break;
            }
            println!("player 1: ");
            let mut p2 = String::new();
            io::stdin().read_line(&mut p2).unwrap();

            die1 = roll(die2);
        }
    }
}

pub fn roll(num: i32) -> i32 {
    let mut rng = rand::thread_rng();

    let points = rng.gen_range(1..num);

    points
}
