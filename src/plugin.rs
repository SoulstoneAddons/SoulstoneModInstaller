use std::io::Write;
use crate::config::USER_AGENT;
use crate::github_releases::{GithubReleases, Release};

pub struct Plugin {
    pub name: String,
    pub url: String,
    pub repo: String,
    pub description: String,
}

pub enum PluginError {
    ReqwestError(reqwest::Error),
    SerdeError(serde_json::Error),
    ZipError(zip::result::ZipError),
    IoError(std::io::Error),
    PluginError(String),
}

// implement display trait for custom error type
impl std::fmt::Display for PluginError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            PluginError::ReqwestError(e) => write!(f, "Reqwest Error: {}", e),
            PluginError::SerdeError(e) => write!(f, "Serde Error: {}", e),
            PluginError::PluginError(e) => write!(f, "Plugin Error: {}", e),
            PluginError::ZipError(e) => write!(f, "Zip Error: {}", e),
            PluginError::IoError(e) => write!(f, "IO Error: {}", e),
        }
    }
}

impl Plugin {

    pub fn download(&self, path: &str) -> Result<(), PluginError> {
        let path = format!("{}/bepinex/plugins", path);

        // the url is the html_url from the github api
        // get the latest release from url + /releases/latest
        let url = format!("{}/releases", self.url);
        let response = reqwest::blocking::Client::new()
            .get(&url)
            .header("User-Agent", USER_AGENT)
            .send()
            .map_err(PluginError::ReqwestError)?;

        // check if status is 403
        if response.status() == 403 {
            println!("Rate limit exceeded, please wait a few minutes and try again.");
            return Err(PluginError::PluginError("Rate limit exceeded".to_string()));
        }

        //  error decoding response body: invalid type: map, expected a string at line 2 column 2
        let response = response.text().map_err(PluginError::ReqwestError)?;

        // invalid type: map, expected a string at line 2 column 2

        let response = serde_json::from_str::<GithubReleases>(&response);
        if response.is_err() {
            return Err(PluginError::SerdeError(response.err().unwrap()));
        }

        let response = response.unwrap();

        // get first asset from release that contains the name 'Skada'
        let first_release = response
            .first()
            .unwrap();

        let asset = first_release
            .assets
            .first();

        if asset.is_none() {
            return Err(PluginError::PluginError("No asset found".to_string()));
        }

        let asset = asset.unwrap();

        // download asset
        println!("Found Release");
        let response = reqwest::blocking::Client::new()
            .get(&asset.browser_download_url)
            .header("User-Agent", USER_AGENT)
            .send()
            .map_err(PluginError::ReqwestError)?;

        // make sure path ends with .dll or .zip
        if !asset.name.ends_with(".dll") && !asset.name.ends_with(".zip") {
            return Err(PluginError::PluginError("Asset is not a dll or zip file".to_string()));
        }

        if !std::path::Path::new(&path).exists() {
            std::fs::create_dir_all(&path).unwrap();
        }

        let file_path = format!("{}/{}", path, asset.name);
        // save asset to path
        let mut file = std::fs::File::create(&file_path).unwrap();
        file.write_all(&response.bytes().unwrap()).unwrap();


        // unzip asset if it is a zip file
        if asset.name.ends_with(".zip") {
            let mut archive = zip::ZipArchive::new( std::fs::File::open(&file_path).unwrap()).unwrap();
            archive.extract(path).map_err(PluginError::ZipError)?;
            // delete zip file
            std::fs::remove_file(file_path).unwrap();
        }


        return Ok(());
    }
}