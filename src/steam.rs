use std::path::Path;
use winreg::enums::*;
use winreg::RegKey;

pub(crate) struct Steam;

#[derive(Debug)]
pub struct SteamGame {
    pub id: String,
    pub name: String,
    pub path: String,
}

impl Steam {
    pub fn get_library_folders(path: &str) -> Option<Vec<String>> {
        // read libraryfolders.vdf
        let library_folders_path = Path::new(path).join("steamapps/libraryfolders.vdf");
        let library_folders_file = std::fs::read_to_string(library_folders_path);
        if library_folders_file.is_err() {
            return None;
        }
        let library_folders_file = library_folders_file.unwrap();

        // parse libraryfolders.vdf
        let mut library_folders = Vec::new();
        for line in library_folders_file.lines() {
            // parse library folder path
            if line.contains("\"path\"") {
                // get path
                let path = line
                    .split("\"")
                    .nth(3)
                    .unwrap_or("")
                    .replace("\\\\", "\\");
                // add library folder path to list
                if !path.is_empty() {
                    library_folders.push(path);
                }
            }
        }

        // return library folders
        return Some(library_folders);
    }

    pub fn iterate_games(path: &str) -> Option<Vec<SteamGame>> {
        let mut games = Vec::new();
        let folders = Steam::get_library_folders(path);
        if folders.is_none() {
            return None;
        }

        for folder in folders.unwrap() {
            let folder_path = Path::new(&folder).join("steamapps");
            let apps = folder_path.read_dir();
            if apps.is_err() {
                continue;
            }
            for app in apps.unwrap() {
                if app.is_err() {
                    continue;
                }
                let app = app.unwrap();
                let app_path = app.path();
                if app_path.is_dir() {
                    continue;
                }
                let app_name = app_path.file_name().unwrap().to_str().unwrap();
                if !app_name.starts_with("appmanifest_") {
                    continue;
                }
                let app_id = app_name.replace("appmanifest_", "");
                let app_id = app_id.replace(".acf", "");
                let app_id = app_id.parse::<u32>();
                if app_id.is_err() {
                    continue;
                }
                let app_id = app_id.unwrap();
                let app_id = app_id.to_string();
                let app_path = folder_path.join("appmanifest_".to_owned() + &app_id + ".acf");
                let app_file = std::fs::read_to_string(app_path);
                if app_file.is_err() {
                    continue;
                }
                let app_file = app_file.unwrap();
                let mut app_name = String::new();
                let mut app_path = String::new();
                for line in app_file.lines() {
                    if line.contains("\"name\"") {
                        app_name = line.split("\"").nth(3).unwrap_or("").to_string();
                    }
                    if line.contains("\"installdir\"") {
                        app_path = line.split("\"").nth(3).unwrap_or("").to_string();
                    }
                }
                if app_name.is_empty() || app_path.is_empty() {
                    continue;
                }
                let app_path = folder_path.join("common").join(app_path);
                let app_path = app_path.to_str().unwrap().to_string();
                let game = SteamGame {
                    id: app_id,
                    name: app_name,
                    path: app_path,
                };
                games.push(game);
            }
        }

        return Some(games);
    }

    #[cfg(target_os = "windows")]
    pub fn get_steam_path() -> Option<String> {
        // open registry key
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        // find steam path for both 32 and 64 bit
        let steam_key = hklm.open_subkey_with_flags(
            r"SOFTWARE\WOW6432Node\Valve\Steam",
            KEY_READ | KEY_WOW64_32KEY,
        );
        // if steam key not found, try 64 bit
        let steam_key = if steam_key.is_err() {
            hklm.open_subkey_with_flags(
                r"SOFTWARE\Valve\Steam",
                KEY_READ | KEY_WOW64_64KEY,
            )
        } else {
            steam_key
        };
        if steam_key.is_err() {
            return None;
        }

        return if let Ok(key) = steam_key.unwrap().get_value("InstallPath") {
            Some(key)
        } else {
            None
        }
    }

    #[cfg(target_os = "linux")]
    pub fn get_steam_path() -> Option<String> {
        // get home directory
        let home_dir = dirs::home_dir();
        if home_dir.is_none() {
            return None;
        }
        let home_dir = home_dir.unwrap();

        // get steam path
        let steam_path = home_dir.join(".steam/steam");
        if !steam_path.exists() {
            return None;
        }

        return Some(steam_path.to_str().unwrap().to_string());
    }


}

