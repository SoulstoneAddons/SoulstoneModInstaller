use crate::github::{GithubRepositories, Repository};

pub struct Plugins;

impl Plugins {

    fn get_repositories() -> Result<GithubRepositories, reqwest::Error> {
        let response = reqwest::blocking::Client::new()
            .get("https://api.github.com/orgs/SoulstoneAddons/repos")
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/111.0.0.0 Safari/537.36")
            .send()?;

        let response = response.json::<GithubRepositories>()?;

        return Ok(response);
    }

    pub fn get_plugins() -> Result<Vec<Repository>, reqwest::Error> {
        let plugins = Plugins::get_repositories()?;
        return Ok(plugins
            .into_iter()
            .filter(|x| !x.private && x.topics.contains(&"plugin".to_string()))
            .collect::<Vec<_>>());
    }
}