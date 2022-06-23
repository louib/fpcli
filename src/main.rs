//! This is the binary crate for the `fpcli` executable.
//! For a Flatpak library for Rust, see [flatpak-rs](https://crates.io/crates/flatpak-rs)
//! To get the list of available commands, run `fpcli -h`.
use std::env;
use std::fs;
use std::path;

use clap::{AppSettings, Parser, Subcommand};
use flatpak_rs::application::FlatpakApplication;
use flatpak_rs::format::FlatpakManifestFormat;
use flatpak_rs::module::{FlatpakModule, FlatpakModuleItem};
use flatpak_rs::source::{FlatpakSource, FlatpakSourceItem, FlatpakSourceType};

mod bare_install;
mod utils;

/// A CLI app for Flatpak manifests.
#[derive(Parser)]
#[clap(name = "fpcli")]
#[clap(version = env!("CARGO_PKG_VERSION"))]
#[clap(about = "A CLI app for Flatpak manifests.", long_about = None)]
struct Fpcli {
    #[clap(subcommand)]
    command: SubCommand,
}

#[derive(Subcommand)]
enum SubCommand {
    /// Formats a Flatpak manifest.
    #[clap(setting(AppSettings::ArgRequiredElseHelp))]
    Lint {
        /// The path of the manifest to lint.
        path: String,
        /// Only check the manifest for formatting issues.
        #[clap(long, short)]
        check: bool,
    },
    /// List all the Flatpak manifests in a specific directory.
    Ls {
        /// The path of the directory to traverse.
        path: String,
    },
    /// Get all the urls contained in a manifest.
    #[clap(name = "get-urls")]
    GetUrls {
        /// The path of the manifest to parse.
        path: String,
    },
    /// Get the type of the manifest
    #[clap(name = "get-type")]
    GetType {
        /// The path of the manifest parse
        path: String,
    },
    /// Converts a manifest. The manifest must be a valid
    /// Flatpak manifest.
    #[clap(setting(AppSettings::ArgRequiredElseHelp))]
    Convert {
        /// The path of the manifest to convert.
        path: String,
        /// The format to convert the manifest to.
        #[clap(name = "format")]
        format_name: String,
    },
    /// Parse a Flatpak manifest.
    #[clap(setting(AppSettings::ArgRequiredElseHelp))]
    Parse {
        /// The path of the manifest to parse.
        path: String,
    },
    /// Resolve all the imported manifests in a manifest file.
    #[clap(setting(AppSettings::ArgRequiredElseHelp))]
    Resolve {
        /// The path of the manifest to resolve.
        path: String,
        /// Only check that the manifests can be resolved.
        #[clap(long, short)]
        check: bool,
    },
    /// Converts a URL to its reverse DNS equivalent.
    #[clap(setting(AppSettings::ArgRequiredElseHelp))]
    ToReverseDNS {
        /// The URL to convert to reverse DNS.
        url: String,
    },
    /// Test if a file path uses a reverse DNS ID.
    #[clap(setting(AppSettings::ArgRequiredElseHelp))]
    IsReverseDNS {
        /// The path of the file to test.
        path: String,
    },
    /// Print the modules of a manifest in a tree-like structure.
    #[clap(setting(AppSettings::ArgRequiredElseHelp))]
    Tree {
        /// The path of the manifest to use.
        path: String,
        /// Also resolve the imported manifests.
        #[clap(long, short)]
        resolve: bool,
        /// Do not print modules deeper than this
        #[clap(long, short)]
        max_depth: Option<i64>,
    },
    /// Creates a new application manifest for the current project.
    Bootstrap {},
}

fn main() -> std::process::ExitCode {
    let args = Fpcli::parse();

    match &args.command {
        SubCommand::Ls { path } => {
            // FIXME right now we don't use the path parameter.
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
        }
        SubCommand::GetUrls { path } => {
            // TODO do some validations on the file path before trying to parse it.
            let mut all_urls: Vec<String> = vec![];

            if let Ok(flatpak_application) = FlatpakApplication::load_from_file(path.to_string()) {
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

            if let Ok(flatpak_module) = FlatpakModule::load_from_file(path.to_string()) {
                for url in flatpak_module.get_all_urls() {
                    println!("{}", url);
                }
            }
        }
        SubCommand::GetType { path } => {
            if !path::Path::new(&path).is_file() {
                eprintln!("{} is not a file.", path);
                return std::process::ExitCode::FAILURE;
            }
            if FlatpakApplication::load_from_file(path.to_string()).is_ok() {
                println!("application");
            };
            if FlatpakModule::load_from_file(path.to_string()).is_ok() {
                println!("module");
            };
            if FlatpakSource::load_from_file(path.to_string()).is_ok() {
                // TODO should we differentiate with 1 source VS multiple sources?
                println!("source");
            };
            eprintln!("{} is not a Flatpak manifest.", path);
            return std::process::ExitCode::SUCCESS;
        }
        SubCommand::Convert { path, format_name } => {
            // TODO we should also try to parse the file as a module manifest or as a source manifest!
            let mut flatpak_application = match FlatpakApplication::load_from_file(path.to_string())
            {
                Ok(m) => m,
                Err(e) => {
                    eprintln!("Could not parse manifest file at {}: {}.", path, e);
                    return std::process::ExitCode::FAILURE;
                }
            };

            // This is not optimal. Maybe we need a standalone function for that.
            let format = match FlatpakManifestFormat::from_path(format_name) {
                Some(f) => f,
                None => panic!("Invalid destination format {}.", format_name),
            };

            flatpak_application.format = format;

            let application_dump = match flatpak_application.dump() {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("Could not dump manifest: {}.", e);
                    return std::process::ExitCode::FAILURE;
                }
            };
            println!("{}", application_dump);
        }
        SubCommand::Lint { path, check } => {
            // TODO we should also try to parse the file as a module manifest or as a source manifest!
            let flatpak_application = match FlatpakApplication::load_from_file(path.to_string()) {
                Ok(m) => m,
                Err(e) => {
                    eprintln!("Could not parse manifest file at {}: {}.", path, e);
                    return std::process::ExitCode::FAILURE;
                }
            };

            let initial_content = match fs::read_to_string(path) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Could not read file {}: {}!", &path, e);
                    return std::process::ExitCode::FAILURE;
                }
            };

            let application_dump = match flatpak_application.dump() {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("Could not dump manifest: {}.", e);
                    return std::process::ExitCode::FAILURE;
                }
            };

            if *check {
                if application_dump == initial_content {
                    println!("The file is formatted correctly.");
                    return std::process::ExitCode::FAILURE;
                } else {
                    panic!("There are formatting issues with the file.");
                }
            }

            if let Err(e) = fs::write(path::Path::new(&path), application_dump) {
                panic!("could not write file {}: {}.", path, e);
            };
        }
        SubCommand::Parse { path } => {
            // TODO we should also try to parse the file as a module manifest or as a source manifest!
            let flatpak_application = match FlatpakApplication::load_from_file(path.to_string()) {
                Ok(m) => m,
                Err(e) => {
                    eprintln!("Could not parse manifest file at {}: {}.", path, e);
                    return std::process::ExitCode::FAILURE;
                }
            };

            println!(
                "Parsed Flatpak application manifest for app {}.",
                flatpak_application.get_id()
            );
        }
        SubCommand::ToReverseDNS { url } => {
            println!("{}", flatpak_rs::reverse_dns::from_url(url))
        }
        SubCommand::IsReverseDNS { path } => {
            println!("{}", flatpak_rs::reverse_dns::is_reverse_dns(path))
        }
        SubCommand::Resolve { path, check } => {
            // TODO we should also try to parse the file as a module manifest here.
            let mut flatpak_application = match FlatpakApplication::load_from_file(path.to_string())
            {
                Ok(m) => m,
                Err(e) => {
                    eprintln!("Could not parse manifest file at {}: {}.", path, e);
                    return std::process::ExitCode::FAILURE;
                }
            };

            resolve_application(path, &mut flatpak_application);

            if *check {
                return std::process::ExitCode::SUCCESS;
            }

            let application_dump = flatpak_application.dump().unwrap();
            match fs::write(path::Path::new(&path), application_dump) {
                Ok(content) => content,
                Err(e) => {
                    eprintln!("could not write file {}: {}.", &path, e);
                    return std::process::ExitCode::FAILURE;
                }
            };
        }
        SubCommand::Tree {
            path,
            resolve,
            max_depth,
        } => {
            // TODO we should also try to parse the file as a module manifest here.
            let mut flatpak_application = match FlatpakApplication::load_from_file(path.to_string())
            {
                Ok(m) => m,
                Err(e) => {
                    eprintln!("Could not parse manifest file at {}: {}.", path, e);
                    return std::process::ExitCode::FAILURE;
                }
            };

            if *resolve {
                resolve_application(path, &mut flatpak_application);
            }

            // TODO add a maximum depth option.

            println!("{}", flatpak_application.get_id());
            print_modules(&flatpak_application.modules, 0, max_depth.unwrap_or(1000));
        }
        SubCommand::Bootstrap {} => {
            let mut flatpak_application = FlatpakApplication::default();
            flatpak_application.format = FlatpakManifestFormat::YAML;
            flatpak_application.id = "org.example.appName".to_string();
            flatpak_application.runtime = "org.gnome.Platform".to_string();
            flatpak_application.runtime_version = "41".to_string();
            flatpak_application.sdk = "org.gnome.Sdk".to_string();

            flatpak_application
                .finish_args
                .push("--filesystem=home".to_string());
            flatpak_application
                .finish_args
                .push("--socket=x11".to_string());
            flatpak_application
                .finish_args
                .push("--socket=wayland".to_string());

            let mut current_module = FlatpakModule::default();
            let mut current_source = FlatpakSource::default();

            current_source.r#type = Some(FlatpakSourceType::Git);
            current_source.path = Some("./".to_string());
            current_module
                .sources
                .push(FlatpakSourceItem::Description(current_source));

            flatpak_application
                .modules
                .push(FlatpakModuleItem::Description(current_module));

            println!("{}", flatpak_application.dump().unwrap());
        }
    };
    std::process::ExitCode::SUCCESS
}

pub fn resolve_application(path: &str, application: &mut FlatpakApplication) {
    let mut new_base_path = match path::Path::new(path).parent() {
        Some(b) => b.to_str().unwrap(),
        None => "",
    };
    application.modules = resolve_modules(new_base_path, &application.modules);
    println!("Resolved modules for {}.", application.get_id());
    // FIXME we should actually also resolve the imported sources here.
}

pub fn print_modules(module_items: &Vec<FlatpakModuleItem>, depth: i64, max_depth: i64) {
    if depth > max_depth {
        return;
    }

    let mut indent = "".to_string();
    for n in 0..depth {
        indent = format!("{}{}", indent, "  ");
    }

    for module in module_items {
        match module {
            FlatpakModuleItem::Description(m) => {
                println!("{}↪ {}", indent, m.name);
                print_modules(&m.modules, depth + 1, max_depth);
            }
            FlatpakModuleItem::Path(p) => {
                println!("{}↪ {}", indent, p);
            }
        }
    }
}

pub fn resolve_modules(
    base_path: &str,
    module_items: &Vec<FlatpakModuleItem>,
) -> Vec<FlatpakModuleItem> {
    let mut response: Vec<FlatpakModuleItem> = vec![];
    for module_item in module_items {
        match module_item {
            FlatpakModuleItem::Path(p) => {
                let mut new_base_path = match path::Path::new(&p).parent() {
                    Some(b) => b.to_str().unwrap(),
                    None => "",
                };
                let full_file_path = format!("{}/{}", base_path, p);
                let mut module = FlatpakModule::load_from_file(full_file_path).unwrap();
                module.modules = resolve_modules(new_base_path, &module.modules);
                response.push(FlatpakModuleItem::Description(module));
            }
            FlatpakModuleItem::Description(m) => {
                let mut module = m.clone();
                module.modules = resolve_modules(base_path, &module.modules);
                response.push(FlatpakModuleItem::Description(module));
            }
        };
    }
    response
}
