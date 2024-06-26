//! This is the binary crate for the `fpcli` executable.
//! For a Flatpak library for Rust, see [flatpak-rs](https://crates.io/crates/flatpak-rs)
//! To get the list of available commands, run `fpcli -h`.
use std::env;
use std::ffi::OsString;
use std::fs;
use std::path;

use clap::{AppSettings, Parser, Subcommand};
use flatpak_rs::application::FlatpakApplication;
use flatpak_rs::format::FlatpakManifestFormat;
use flatpak_rs::manifest_type::FlatpakManifestType;
use flatpak_rs::module::{FlatpakModule, FlatpakModuleItem};
use flatpak_rs::source::{FlatpakSource, FlatpakSourceItem, FlatpakSourceType};

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
    /// Install all the modules in a manifest. The manifest has to be an application
    /// manifest or a module manifest.
    #[clap(setting(AppSettings::ArgRequiredElseHelp))]
    Install {
        /// The path of the application or module manifest to install.
        path: String,

        /// Do not install the modules in a flatpak container. Install them
        /// on the host system instead.
        #[clap(long, short)]
        bare: bool,

        /// Only print the install instructions to stdout.
        #[clap(long, short)]
        print: bool,
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
        /// Only include the URLs for a specific source type.
        source_type: Option<String>,
        /// Also includes the mirror urls
        #[clap(long, short)]
        mirror_urls: bool,
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
    /// Add a module to a Flatpak manifest. The target flatpak manifest
    /// must be a Flatpak application manifest or a Flatpak module
    /// manifest. By default, the module is imported by path.
    #[clap(setting(AppSettings::ArgRequiredElseHelp))]
    AddModule {
        /// The path of the manifest to add the module to.
        manifest_path: String,
        /// The path of the module to add to the manifest.
        module_path: String,
        /// Inline the module instead of importing by path.
        #[clap(long, short)]
        inline: bool,
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
    /// Creates a new manifest from the available information.
    Bootstrap {
        /// The type of manifest to bootstrap. This will
        /// default to an application manifest.
        #[clap(long, short)]
        manifest_type: Option<String>,

        /// A build system, in the case of an application or a module.
        build_system: Option<String>,

        /// A url to bootstrap from.
        #[clap(long, short)]
        url: Option<String>,
    },
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
        SubCommand::GetUrls {
            path,
            source_type,
            mirror_urls,
        } => {
            // TODO do some validations on the file path before trying to parse it.
            // TODO we should probably resolve the manifest completely? Or at least offer an option
            // to also resolve the manifest.
            let mut included_source_types: Option<Vec<FlatpakSourceType>> = None;
            if let Some(source_type) = source_type {
                included_source_types =
                    Some(vec![FlatpakSourceType::from_string(source_type).unwrap()]);
            }

            if let Ok(flatpak_application) = FlatpakApplication::load_from_file(path.to_string()) {
                for url in flatpak_application.get_urls(*mirror_urls, included_source_types.clone())
                {
                    println!("{}", url);
                }

                return std::process::ExitCode::SUCCESS;
            }

            if let Ok(flatpak_module) = FlatpakModule::load_from_file(path.to_string()) {
                for url in flatpak_module.get_urls(*mirror_urls, included_source_types.clone()) {
                    println!("{}", url);
                }

                return std::process::ExitCode::SUCCESS;
            }

            if let Ok(flatpak_source) = FlatpakSource::load_from_file(path.to_string()) {
                for source in flatpak_source {
                    for url in source.get_urls(*mirror_urls, included_source_types.clone()) {
                        println!("{}", url);
                    }
                }

                return std::process::ExitCode::SUCCESS;
            }

            eprintln!("File at {} is not a Flatpak manifest.", path);
            return std::process::ExitCode::FAILURE;
        }
        SubCommand::GetType { path } => {
            if !path::Path::new(&path).is_file() {
                eprintln!("{} is not a file.", path);
                return std::process::ExitCode::FAILURE;
            }
            if FlatpakApplication::load_from_file(path.to_string()).is_ok() {
                println!("application");
                return std::process::ExitCode::SUCCESS;
            };
            if FlatpakModule::load_from_file(path.to_string()).is_ok() {
                println!("module");
                return std::process::ExitCode::SUCCESS;
            };
            if FlatpakSource::load_from_file(path.to_string()).is_ok() {
                // TODO should we differentiate with 1 source VS multiple sources?
                println!("source");
                return std::process::ExitCode::SUCCESS;
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
        SubCommand::Install { path, bare, print } => {
            let mut modules: Vec<FlatpakModule> = vec![];

            if let Ok(mut flatpak_app) = FlatpakApplication::load_from_file(path.to_string()) {
                resolve_application(path, &mut flatpak_app);

                for module in flatpak_app.get_all_modules_recursively() {
                    let module_description = match module {
                        FlatpakModuleItem::Path(_) => {
                            panic!("There should be no module defined as path after resolving.")
                        }
                        FlatpakModuleItem::Description(d) => d,
                    };
                    modules.push(module_description.clone());
                }
            }

            if let Ok(flatpak_module) = FlatpakModule::load_from_file(path.to_string()) {
                let mut resolved_modules =
                    resolve_modules(path, &vec![FlatpakModuleItem::Description(flatpak_module)]);

                for module in resolved_modules {
                    let module_description = match module {
                        FlatpakModuleItem::Path(_) => {
                            panic!("There should be no module defined as path after resolving.")
                        }
                        FlatpakModuleItem::Description(d) => d,
                    };
                    modules.push(module_description.to_owned());
                    for inner_module in module_description.get_all_modules_recursively() {
                        let inner_module_description = match inner_module {
                            FlatpakModuleItem::Path(_) => {
                                panic!("There should be no module defined as path after resolving.")
                            }
                            FlatpakModuleItem::Description(d) => d,
                        };
                        modules.push(inner_module_description.to_owned());
                    }
                }
            }

            if !bare || !print {
                panic!("Only --print and --bare is supported at the moment.");
            }

            for module in modules {
                let mut args: Vec<OsString> = vec![];

                eprintln!("Installing module {}", module.name);
                for command in module.get_commands(args, true, "", "", Some(""), 0) {
                    print!("{}", command.get_program().to_str().unwrap());
                    for arg in command.get_args() {
                        print!(" {}", arg.to_str().unwrap());
                    }
                    println!("");
                }
            }
        }
        SubCommand::Lint { path, check } => {
            if let Ok(flatpak_application) = FlatpakApplication::load_from_file(path.to_string()) {
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

                return std::process::ExitCode::SUCCESS;
            }

            if let Ok(flatpak_module) = FlatpakModule::load_from_file(path.to_string()) {
                let initial_content = match fs::read_to_string(path) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("Could not read file {}: {}!", &path, e);
                        return std::process::ExitCode::FAILURE;
                    }
                };

                let module_dump = match flatpak_module.dump() {
                    Ok(d) => d,
                    Err(e) => {
                        eprintln!("Could not dump manifest: {}.", e);
                        return std::process::ExitCode::FAILURE;
                    }
                };

                if *check {
                    if module_dump == initial_content {
                        println!("The file is formatted correctly.");
                        return std::process::ExitCode::FAILURE;
                    } else {
                        panic!("There are formatting issues with the file.");
                    }
                }

                if let Err(e) = fs::write(path::Path::new(&path), module_dump) {
                    panic!("could not write file {}: {}.", path, e);
                };

                return std::process::ExitCode::SUCCESS;
            }

            if let Ok(flatpak_sources) = FlatpakSource::load_from_file(path.to_string()) {
                panic!("Lint Flatpak source manifests is not yet supported :(");
            }

            panic!("Could not parse Flatpak manifest at {}.", path);
        }
        SubCommand::Parse { path } => {
            match FlatpakApplication::load_from_file(path.to_string()) {
                Ok(a) => {
                    println!("Parsed Flatpak application manifest for {}.", a.get_id());
                    return std::process::ExitCode::SUCCESS;
                }
                Err(e) => {
                    eprintln!("Could not parse Flatpak application from {}: {}.", path, e);
                }
            };

            match FlatpakModule::load_from_file(path.to_string()) {
                Ok(module) => {
                    println!("Parsed Flatpak module manifest for {}.", module.name);
                    return std::process::ExitCode::SUCCESS;
                }
                Err(e) => {
                    eprintln!("Could not parse Flatpak module from {}: {}.", path, e);
                }
            };

            match FlatpakSource::load_from_file(path.to_string()) {
                Ok(sources) => {
                    println!("Parsed {} Flatpak source(s) from file.", sources.len());
                    return std::process::ExitCode::SUCCESS;
                }
                Err(e) => {
                    eprintln!("Could not parse Flatpak sources from {}: {}.", path, e);
                }
            };

            return std::process::ExitCode::FAILURE;
        }
        SubCommand::ToReverseDNS { url } => {
            println!("{}", flatpak_rs::reverse_dns::from_url(url))
        }
        SubCommand::IsReverseDNS { path } => {
            println!("{}", flatpak_rs::reverse_dns::is_reverse_dns(path))
        }
        SubCommand::AddModule {
            manifest_path,
            module_path,
            inline,
        } => {
            if *inline {
                panic!("The inline option is not implemented yet!");
            }
            match FlatpakApplication::load_from_file(manifest_path.to_string()) {
                Ok(mut app) => {
                    println!("Parsed Flatpak application manifest for {}.", app.get_id());

                    app.modules
                        .push(FlatpakModuleItem::Path(manifest_path.to_string()));

                    let app_dump = app.dump().unwrap();
                    if let Err(e) = fs::write(path::Path::new(&manifest_path), app_dump) {
                        eprintln!("could not write file {}: {}.", &manifest_path, e);
                        return std::process::ExitCode::FAILURE;
                    }

                    return std::process::ExitCode::SUCCESS;
                }
                Err(e) => {
                    eprintln!(
                        "Could not parse Flatpak application from {}: {}.",
                        manifest_path, e
                    );
                }
            };

            match FlatpakModule::load_from_file(manifest_path.to_string()) {
                Ok(mut module) => {
                    module
                        .modules
                        .push(FlatpakModuleItem::Path(manifest_path.to_string()));

                    println!("Parsed Flatpak module manifest for {}.", module.name);
                    return std::process::ExitCode::SUCCESS;
                }
                Err(e) => {
                    eprintln!(
                        "Could not parse Flatpak module from {}: {}.",
                        manifest_path, e
                    );
                }
            };

            eprintln!("Could not parse Flatpak manifest at {}", manifest_path);
            return std::process::ExitCode::FAILURE;
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
            if let Err(e) = fs::write(path::Path::new(&path), application_dump) {
                eprintln!("could not write file {}: {}.", &path, e);
                return std::process::ExitCode::FAILURE;
            };

            return std::process::ExitCode::SUCCESS;
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
        SubCommand::Bootstrap {
            manifest_type,
            build_system,
            url,
        } => {
            if let Some(manifest_type) = manifest_type {
                let manifest_type = match FlatpakManifestType::from_string(&manifest_type) {
                    Ok(t) => t,
                    Err(e) => {
                        eprintln!("Invalid manifest type {:?}.", manifest_type);
                        return std::process::ExitCode::FAILURE;
                    }
                };

                match manifest_type {
                    FlatpakManifestType::Application => {
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

                        let mut default_module = get_default_module(url.to_owned());
                        flatpak_application
                            .modules
                            .push(FlatpakModuleItem::Description(default_module));

                        println!("{}", flatpak_application.dump().unwrap());
                    }
                    FlatpakManifestType::Module => {
                        let mut default_module = get_default_module(url.to_owned());
                        println!("{}", default_module.dump().unwrap());
                    }
                    FlatpakManifestType::Source => {
                        // let mut default_source = get_default_source(url.to_owned());
                        // println!("{}", default_source.dump().unwrap());
                        eprintln!("Bootstrapping a source manifest is not supported yet.");
                    }
                };
            }
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

pub const DEFAULT_GIT_BRANCH: &str = "master";

pub fn get_default_source(url: Option<String>) -> FlatpakSource {
    let mut default_source = FlatpakSource::default();

    if let Some(url) = url {
        if url.ends_with(".git") {
            default_source.r#type = Some(FlatpakSourceType::Git);
            default_source.branch = Some(DEFAULT_GIT_BRANCH.to_string());
        } else {
            default_source.r#type = Some(FlatpakSourceType::Archive);
        }
        default_source.url = Some(url.clone());
    } else {
        default_source.r#type = Some(FlatpakSourceType::Dir);
        default_source.path = Some("./".to_string());
    }
    default_source
}

pub fn get_default_module(url: Option<String>) -> FlatpakModule {
    let mut default_module = FlatpakModule::default();
    let default_source = get_default_source(url);
    default_module
        .sources
        .push(FlatpakSourceItem::Description(default_source.clone()));

    if default_source.url.is_none() {
        return default_module;
    }

    if default_source.get_type() == Some(FlatpakSourceType::Git) {
        if let Some(project_name) =
            get_project_name_from_git_url(default_source.url.as_ref().unwrap().to_string())
        {
            default_module.name = format!("{}.{}", project_name, DEFAULT_GIT_BRANCH);
        } else {
            default_module.name = format!("project-name.{}", DEFAULT_GIT_BRANCH);
        }
    } else if default_source.get_type() == Some(FlatpakSourceType::Archive) {
        if let Some(project_name) =
            flatpak_rs::archive::get_project_name_from_url(default_source.url.as_ref().unwrap())
        {
            default_module.name = format!("{}.archive", project_name);
        } else {
            default_module.name = format!("project-name.{}", DEFAULT_GIT_BRANCH);
        }
    }
    default_module
}

///```
///let project_name = crate::main::get_project_name_from_git_url(
///  "https://github.com/louib/flatpak-rs.git"
///);
///assert!(project_name.is_some());
///assert_eq!(project_name.unwrap(), "flatpak-rs");
///
///let project_name = crate::main::get_project_name_from_git_url(
///  "git@github.com:louib/flatpak-rs.git"
///);
///assert!(project_name.is_some());
///assert_eq!(project_name.unwrap(), "flatpak-rs");
///```
pub fn get_project_name_from_git_url(url: String) -> Option<String> {
    if !url.ends_with(".git") {
        return None;
    }

    let url = url.replace(".git", "");
    let project_name = url.split("/").last().unwrap();

    Some(project_name.to_string())
}
