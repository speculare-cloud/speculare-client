use super::{Config, PluginInfo, PluginsMap};

use std::{collections::HashMap, io::Error, io::ErrorKind};

pub fn get_plugins(config: &Config) -> Result<PluginsMap, Error> {
    let mut plugins: PluginsMap = HashMap::new();
    let paths = std::fs::read_dir(&config.plugins_path)?;
    trace!("successfully read the plugins folder");
    for path in paths {
        // TODO - Get rid of unsafe unwrap
        let path = path.unwrap();
        debug!("is {:?} a plugin", path.path());
        let lib = match unsafe { libloading::Library::new(path.path()) } {
            Ok(library) => {
                trace!("plugin ({:?}) loaded correctly", path.file_name());
                library
            }
            Err(err_lib) => {
                error!(
                    "plugin ({:?}) failed to load: {:?}",
                    path.file_name(),
                    err_lib
                );
                continue;
            }
        };
        // TODO - Get rid of unsafe unwrap
        let info: fn() -> String = *(unsafe { lib.get(b"info") }.unwrap());
        let func: fn() -> Result<String, Error> = *(unsafe { lib.get(b"entrypoint") }.unwrap());
        plugins.insert(info(), PluginInfo { lib, func });
    }
    // Return the PluginsMap if there are some
    // Else return an error (NotFound)
    if !plugins.is_empty() {
        Ok(plugins)
    } else {
        Err(Error::new(ErrorKind::NotFound, "no plugins found"))
    }
}
