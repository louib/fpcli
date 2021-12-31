use std::env;
use std::path;

use flatpak_rs::flatpak_manifest::{
    FlatpakManifest, FlatpakModule, FlatpakModuleDescription, FlatpakSourceDescription,
};

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

    if command_name == "get-urls" {
        if args.len() <= 2 {
            panic!("Please provide a file path to parse.");
        }
        let file_path = args[2].clone();

        // TODO do some validations on the file path before trying to parse it.
        let mut all_urls: Vec<String> = vec![];

        if let Ok(flatpak_manifest) = FlatpakManifest::load_from_file(file_path.to_string()) {
            for module in flatpak_manifest.get_all_modules_recursively() {
                let module_description = match module {
                    FlatpakModule::Description(d) => d,
                    FlatpakModule::Path(_) => continue,
                };
                for url in module_description.get_all_urls() {
                    println!("{}", url);
                }
            }
        }

        if let Ok(flatpak_module) = FlatpakModuleDescription::load_from_file(file_path.to_string())
        {
            for url in flatpak_module.get_all_urls() {
                println!("{}", url);
            }
        }
    }

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

            if let Ok(flatpak_manifest) = FlatpakManifest::load_from_file(file_path.to_string()) {
                println!("Flatpak application at {}.", &file_path);
            }
        }
        return;
    }

    // I should be able to list the valid command names here,
    // or this should have been handled earlier?
    panic!("Invalid command name {}", command_name);
}
