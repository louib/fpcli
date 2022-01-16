use flatpak_rs::build_system::FlatpakBuildSystem;
use flatpak_rs::module::FlatpakModuleDescription;

pub fn bare_install(module: &FlatpakModuleDescription) -> Result<(), String> {
    // TODO create a temp directory with the name of the module.
    // If it's a git repo, check to see if we already cloned it.
    // if it's an archive, check if we already downloaded it.
    //
    match &module.buildsystem {
        FlatpakBuildSystem::Meson => {
            // TODO check if a meson.build file exists
        }
        FlatpakBuildSystem::CMake => {}
        FlatpakBuildSystem::Simple => {
            if module.build_commands.is_empty() {
                return Err("Buildsystem simple requires build-commands.".to_string());
            }
        }
        FlatpakBuildSystem::Autotools => {}
        FlatpakBuildSystem::QMake => {
            return Err(
                "qmake not implemented yet. Open an issue at https://github.com/louib/fpcli/issues"
                    .to_string(),
            );
        }
        FlatpakBuildSystem::CMakeNinja => {
            return Err("cmake-ninja not implemented yet. Open an issue at https://github.com/louib/fpcli/issues".to_string());
        }
    }

    Ok(())
}
