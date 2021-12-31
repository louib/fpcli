use std::env;

fn main() {
    let mut command_name: String = "".to_string();
    let args: Vec<String> = env::args().collect();
    if args.len() >= 2 {
        command_name = args[1].clone();
    }

    // Not sure this makes sense long term.
    if command_name.len() == 0 {
        println!("No command provided, defaulting to ls");
        command_name = "ls".to_string();
    }
    println!("Executing command {}.", command_name);

    if command_name == "ls" {}

    // I should be able to list the valid command names here,
    // or this should have been handled earlier?
    panic!("Invalid command name {}", command_name);
}
