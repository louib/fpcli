use std::env;
use std::fs;
use std::path;

use flatpak_rs::application::FlatpakApplication;
use flatpak_rs::module::{FlatpakModule, FlatpakModuleItem};

mod bare_install;
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

        if let Ok(flatpak_application) = FlatpakApplication::load_from_file(file_path.to_string()) {
            for module in flatpak_application.get_all_modules_recursively() {
                let module_description = match module {
                    FlatpakModuleItem::Description(d) => d,
                    FlatpakModuleItem::Path(_) => continue,
                };
                for url in module_description.get_all_urls() {
                    println!("{}", url);
                }
            }
        }

        if let Ok(flatpak_module) = FlatpakModule::load_from_file(file_path.to_string()) {
            for url in flatpak_module.get_all_urls() {
                println!("{}", url);
            }
        }

        // TODO Also try to parse for source manifests.
        return;
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

            if let Ok(flatpak_application) =
                FlatpakApplication::load_from_file(file_path.to_string())
            {
                println!("Flatpak application at {}.", &file_path);
            }
        }
        return;
    }

    if command_name == "lint" {
        if args.len() <= 2 {
            panic!("Please provide a file path to parse.");
        }
        let file_path = args[2].clone();

        // TODO we should also try to parse the file as a module manifest or as a source manifest!
        let flatpak_application = match FlatpakApplication::load_from_file(file_path.to_string()) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("Could not parse manifest file at {}: {}.", file_path, e);
                return;
            }
        };

        let application_dump = match flatpak_application.dump() {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Could not dump manifest: {}.", e);
                return;
            }
        };

        if let Err(e) = fs::write(path::Path::new(&file_path), application_dump) {
            panic!("could not write file {}: {}.", file_path, e);
        };

        return;
    }

    // I should be able to list the valid command names here,
    // or this should have been handled earlier?
    panic!("Invalid command name {}", command_name);
}
