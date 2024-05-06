macro_rules! scanln {
    () => {{
        use std::io::Write;
        let mut input = String::new();
        std::io::stdout().flush().expect("Failed to flush stdout");
        std::io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        let input = input.trim().to_string();
        input
    }};
}
macro_rules! print_try_again {
    () => {{
        clear_terminal();
        println!("{}", "Invalid input. Please enter a valid option.".red());
    }};
}
