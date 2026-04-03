#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {{
        println!("[error] {}", format!($($arg)*))
    }};
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {{
        println!("[info] {}", format!($($arg)*))
    }};
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {{
        println!("[debug] {}", format!($($arg)*))
    }};
}