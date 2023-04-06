use std::path::Path;
use crate::config::{BEPINEX_URL, USER_AGENT};
use crate::plugin::Plugin;
use crate::plugins::Plugins;

pub struct Installer {
    path: String,
}

// custom error type
#[derive(Debug)]
pub enum InstallerError {
    IoError(std::io::Error),
    ZipError(zip::result::ZipError),
    ReqwestError(reqwest::Error),
    InstallError(String),
}

pub struct InstallResult {
    pub plugins: Option<Vec<Plugin>>,
    pub installed_bepinex: Option<bool>,
}

// implement display trait for custom error type
impl std::fmt::Display for InstallerError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            InstallerError::IoError(e) => write!(f, "IO Error: {}", e),
            InstallerError::ZipError(e) => write!(f, "Zip Error: {}", e),
            InstallerError::ReqwestError(e) => write!(f, "Reqwest Error: {}", e),
            InstallerError::InstallError(e) => write!(f, "Install Error: {}", e),
        }
    }
}

impl Installer {
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
        }
    }

    pub fn install(&self) -> Result<InstallResult, InstallerError> {
        let mut installed_bepinex: Option<bool> = None;

        // check if installed
        if !self.is_installed() {
            // download bepinex
            self.setup()?;
            installed_bepinex = Some(true);
        }


        // download plugins
        let plugins = self.download_plugins()?;

        return Ok(InstallResult {
            plugins: Some(plugins),
            installed_bepinex,
        });
    }

    fn download_plugins(&self) -> Result<Vec<Plugin>, InstallerError> {
        let plugins = Plugins::get_plugins().map_err(InstallerError::ReqwestError)?;
        let mut installed_plugins = Vec::new();

        for plugin in plugins {
            // prompt do you want to install plugin (Y/N)
            let name = plugin.name.clone();
            println!("Do you want to install {}? (Y/N)", name);
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            let input = input.trim().to_lowercase();
            if input != "y" {
                continue;
            }
            // proceed to download plugin
            println!("Downloading {}...", name);
            let result = plugin.download(self.path.as_str());
            if result.is_err() {
                // get error
                let error = result.err().unwrap();
                println!("Error downloading {}: {}", name, error);
                continue;
            }
            println!("{} downloaded successfully!", name);
            installed_plugins.push(plugin);
        }

        return Ok(installed_plugins);
    }

    fn setup(&self) -> Result<(), InstallerError> {
        // create temp folder
        let temp_path = Path::new(&self.path).join("temp");
        if !temp_path.exists() {
            std::fs::create_dir(&temp_path).map_err(InstallerError::IoError)?;
        }

        // download bepinex
        let bepinex_zip_path = temp_path.join("bepinex.zip");
        let bepinex_zip = reqwest::blocking::Client::new()
            .get(BEPINEX_URL)
            .header("User-Agent", USER_AGENT)
            .send().map_err(InstallerError::ReqwestError)?;

        let bepinex_zip = bepinex_zip.bytes().map_err(InstallerError::ReqwestError)?;
        std::fs::write(&bepinex_zip_path, &bepinex_zip).map_err(InstallerError::IoError)?;

        // extract bepinex
        let bepinex_zip = std::fs::File::open(&bepinex_zip_path).map_err(InstallerError::IoError)?;
        let mut bepinex_zip = zip::ZipArchive::new(bepinex_zip).map_err(InstallerError::ZipError)?;
        bepinex_zip.extract(&temp_path).map_err(InstallerError::ZipError)?;


        // wait for extraction to finish
        std::thread::sleep(std::time::Duration::from_secs(1));

        // remove zip file from temp folder
        std::fs::remove_file(&bepinex_zip_path).map_err(InstallerError::IoError)?;

        // move contents of temp folder to game folder
        let temp_path = Path::new(&self.path).join("temp");
        // move contents of temp folder to game folder which means back one folder
        let game_path = Path::new(&self.path);

        for entry in std::fs::read_dir(&temp_path).map_err(InstallerError::IoError)? {
            let entry = entry.map_err(InstallerError::IoError)?;
            let path = entry.path();
            let file_name = path.file_name().unwrap().to_str().unwrap();
            let game_path = game_path.join(file_name);
            std::fs::rename(&path, &game_path).map_err(InstallerError::IoError)?;
        }


        // delete temp folder
        std::fs::remove_dir_all(&temp_path).map_err(InstallerError::IoError)?;

        return Ok(());
    }

    fn has_ran(&self) -> bool {
        // check if 'bepinex' folder contains 'interop' folder
        let bepinex_path = Path::new(&self.path).join("BepInEx");
        if !bepinex_path.exists() {
            return false;
        }
        // check if 'bepinex' folder contains 'interop' folder
        let bepinex_interop_path = bepinex_path.join("interop");
        if !bepinex_interop_path.exists() {
            return false;
        }

        return true;
    }

    fn is_installed(&self) -> bool {
        // check if 'BepInEx' folder exists
        let bepinex_path = Path::new(&self.path).join("BepInEx");
        // if it does not exist then bepinex is not installed
        if !bepinex_path.exists() {
            return false; // bepinex is not installed
        }

        // check if 'BepInEx' folder contains 'core' folder
        let bepinex_core_path = bepinex_path.join("core");
        // if it does not exist then bepinex is not installed
        if !bepinex_core_path.exists() {
            return false; // bepinex is not installed
        }

        // check if 'BepInEx' folder contains 'plugins' folder
        let bepinex_plugins_path = bepinex_path.join("plugins");
        // if it does not exist then bepinex is not installed
        if !bepinex_plugins_path.exists() {
            return false; // bepinex is not installed
        }

        // check if 'BepInEx' folder contains 'config' folder
        let bepinex_config_path = bepinex_path.join("config");
        // if it does not exist then bepinex is not installed
        if !bepinex_config_path.exists() {
            return false; // bepinex is not installed
        }

        // bepinex is installed
        return true;
    }
}