use std::env;
use std::path;

mod utils;

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

    if command_name == "ls" {
        for file_path in crate::utils::get_all_paths(path::Path::new(".")).unwrap() {
            if !file_path.is_file() {
                continue;
            }

            let file_path = match file_path.to_str() {
                Some(f) => f,
                None => continue,
            };

            if file_path.contains(".git/") {
                continue;
            }

            if let Ok(flatpak_manifest) =
                flatpak_rs::flatpak_manifest::FlatpakManifest::load_from_file(file_path.to_string())
            {
                println!("Flatpak application at {}.", &file_path);
            }
        }
        return;
    }

    // I should be able to list the valid command names here,
    // or this should have been handled earlier?
    panic!("Invalid command name {}", command_name);
}
