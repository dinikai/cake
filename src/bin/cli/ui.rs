macro_rules! work {
    ($($arguments:tt)*) => {
        println!("* {}", format!($($arguments)*));
    };
}

macro_rules! work_error {
    ($($arguments:tt)*) => {
        println!("\x1b[1;31m(!)\x1b[3;22m {}\x1b[0m", format!($($arguments)*));
    };
}

macro_rules! list {
    ($($arguments:tt)*) => {
        println!("- {}", format!($($arguments)*));
    };
}

macro_rules! result {
    ($($arguments:tt)*) => {
        println!(" {}", format!($($arguments)*));
    };
}

macro_rules! result_success {
    ($($arguments:tt)*) => {
        ui::result!("\x1b[32m{}\x1b[0m", format!($($arguments)*));
    };
}

macro_rules! error {
    ($($arguments:tt)*) => {
        println!("\x1b[1;31m(!) error:\x1b[3;22m {}\x1b[0m", format!($($arguments)*));
    };
}

use std::io::{self, Write};

pub(crate) use error;
pub(crate) use list;
pub(crate) use result;
pub(crate) use result_success;
pub(crate) use work;
pub(crate) use work_error;

pub fn confirm(prompt: &str, default_yes: bool, warning: bool) -> bool {
    loop {
        let mut input = String::new();

        let prompt = match warning {
            false => prompt.to_string(),
            true => format!("\x1b[33m{prompt}\x1b[0m"),
        };
        let variants = match default_yes {
            false => String::from("[y/N]"),
            true => String::from("[Y/n]"),
        };

        print!("{prompt} {variants}: ");
        io::stdout().flush().unwrap();

        io::stdin().read_line(&mut input).unwrap();

        match input.trim().to_lowercase().as_str() {
            "y" | "yes" => return true,
            "n" | "no" => return false,
            "" => return default_yes,
            _ => {}
        }
    }
}
