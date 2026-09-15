fn is_debug_mode() -> bool {
    std::env::args().any(|arg| arg == "--debug" || arg == "-d")
}

pub fn debug_print(msg: &str) {
    if is_debug_mode() {
        println!("{}", msg);
    }
}
