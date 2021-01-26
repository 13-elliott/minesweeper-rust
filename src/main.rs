mod controller;
mod model;

use controller::*;
use model::{ErrorKind as ModelErrorKind, MinesweeperModel};
use std::io::stdin;

fn main() {
    let m = MinesweeperModel::new(10, 10, 10).unwrap();
    let c = MinesweeperController::new(m);
    play_game(c);
}

/**
 * Main game logic loop
 */
fn play_game(mut c: MinesweeperController) {
    while c.can_keep_playing() {
        draw_board(c.model(), false);
        let action = get_user_action();
        let (x, y) = get_user_coordinates();
        match action {
            UserAction::Flag => match c.toggle_flag_at(x, y) {
                Ok(added_flag) => {
                    if added_flag {
                        println!("Added a flag at ({}, {})", x, y);
                    } else {
                        println!("Removed a flag from ({}, {})", x, y);
                    }
                }
                Err(ModelErrorKind::OutOfBounds) => {
                    println!("Given coordinates ({}, {}) were not in bounds!", x, y)
                }
                Err(ModelErrorKind::NoOp) => println!("Given coordinates ({}, {}) were already revealed!", x, y),
            },
            UserAction::Reveal => match c.reveal_zone_at(x, y) {
                Err(ModelErrorKind::OutOfBounds) => {
                    println!("Given coordinates were out of bounds!")
                }
                Err(ModelErrorKind::NoOp) => println!("That space was already revealed!"),
                Ok(mine) => {
                    if mine {
                        println!("KA-BOOM!!")
                    }
                }
            },
        }
        println!();
    }
    draw_board(c.model(), true);
    if c.won() {
        println!("Congratulations! You won!")
    } else {
        println!("Sorry! Better luck next time!")
    }
}

enum UserAction {
    Flag,
    Reveal,
}

fn get_user_action() -> UserAction {
    loop {
        let s = get_user_input("(F)lag or (R)eveal?");
        if s.starts_with('f') {
            return UserAction::Flag;
        } else if s.starts_with('r') {
            return UserAction::Reveal;
        } else {
            println!("I didn't understand that!");
        }
    }
}

fn get_coordinate(prompt: &str) -> u32 {
    loop {
        let input = get_user_input(prompt);
        match input.parse() {
            Ok(v) => return v,
            Err(_) => println!("Could not interpret as a non-negative int: \"{}\"", input),
        }
    }
}

fn get_user_coordinates() -> (u32, u32) {
    println!("Note that coordinates are zero-indexed.");
    let x = get_coordinate("Enter x coordinate:");
    let y = get_coordinate("Enter y coordinate:");
    (x, y)
}

fn get_user_input(prompt: &str) -> String {
    let mut input = String::new();
    let stdin = stdin();
    loop {
        println!("{} ", prompt);
        stdin
            .read_line(&mut input)
            .expect("Error reading from stdin!");
        let trimmed = input.trim();
        if trimmed.is_empty() {
            println!("Must not be empty or only whitespace!");
            input.clear();
        } else {
            return trimmed.to_lowercase();
        }
    }
}

/**
 * print the given MinesweeperModel to stdout
 * xray is a flag for debugging purposes, which if true causes all
 * bombs to be displayed regardless of if they have yet been revealed
 */
fn draw_board(model: &MinesweeperModel, xray: bool) {
    let x_item_width = num_digits_b10(model.width() - 1);
    let y_item_width = num_digits_b10(model.height() - 1);

    // print the x-axis
    println!(
        "{0:1$}{2}",
        ' ',
        y_item_width + 1,
        x_axis(model.width(), x_item_width)
    );

    for y in 0..model.height() {
        let mut line = format!("{0:01$} ", y, y_item_width);
        for x in 0..model.width() {
            for _ in 1..x_item_width {
                line.push(' ');
            }
            line.push(if model.is_revealed_at(x, y).unwrap() {
                if model.has_mine_at(x, y).unwrap() {
                    '💥'
                } else {
                    let num_adjacent = model.mines_adjacent_to(x, y).unwrap();
                    if num_adjacent > 0 {
                        std::char::from_digit(num_adjacent, 10).unwrap()
                    } else {
                        '□'
                    }
                }
            } else if xray && model.has_mine_at(x, y).unwrap() {
                if model.is_flagged_at(x, y).unwrap() {
                    '✅'
                } else {
                    '💣'
                }
            } else if model.is_flagged_at(x, y).unwrap() {
                '🚩'
            } else {
                '■'
            });
            line.push(' ');
        }
        // remove final trailing space
        line.pop();
        println!("{}", line);
    }
}

/**
 * given an integer, produces how many digits are needed
 * to represent that number in base-10 (without leading zeros)
 */
fn num_digits_b10(number: u32) -> usize {
    (number as f64).log10().floor() as usize + 1
}

/**
 * Produces the x-axis as a String
 * 
 * model_width is the width of the MinesweeperModel, and number_width is how many
 * digits are needed to represent model_width in base-10 (without leading zeros)
 */
fn x_axis(model_width: u32, number_width: usize) -> String {
    let mut x_axis_string = String::new();
    for x in 0..model_width {
        let num_as_string = format!("{0:01$}", x, number_width);
        x_axis_string.push_str(&num_as_string);
        x_axis_string.push(' ');
    }
    // remove final trailing space
    x_axis_string.pop();

    x_axis_string
}
