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

pub(crate) use error;
pub(crate) use list;
pub(crate) use result;
pub(crate) use result_success;
pub(crate) use work;
pub(crate) use work_error;
