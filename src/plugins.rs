use crate::config::USER_AGENT;
use crate::github_repositories::{GithubRepositories, Repository};
use crate::plugin::Plugin;

pub struct Plugins;

impl Plugins {

    fn get_repositories() -> Result<GithubRepositories, reqwest::Error> {
        // X-Originating-IP: 127.0.0.1
        // X-Forwarded-For: 127.0.0.1
        // X-Remote-IP: 127.0.0.1
        // X-Remote-Addr: 127.0.0.1
        // X-Client-IP: 127.0.0.1
        // X-Host: 127.0.0.1
        // X-Forwared-Host: 127.0.0.1
        let response = reqwest::blocking::Client::new()
            .get("https://api.github.com/orgs/SoulstoneAddons/repos")
            .header("User-Agent", USER_AGENT)
            .send()?;

        // check if status is 403
        if response.status() == 403 {
            println!("Rate limit exceeded, please wait a few minutes and try again.");
            std::process::exit(1);
        }

        let response = response.json::<GithubRepositories>()?;

        return Ok(response);
    }

    pub fn get_plugins() -> Result<Vec<Plugin>, reqwest::Error> {
        let plugins = Plugins::get_repositories()?;
        let plugins = plugins
            .into_iter()
            .filter(|x| !x.private && x.topics.contains(&"plugin".to_string()))
            .collect::<Vec<_>>()
            .iter()
            .map(|x| Plugin {
                name: x.name.clone().unwrap_or("".to_string()),
                description: x.description.clone().unwrap_or("".to_string()),
                url: x.url.clone(),
                repo: x.html_url.clone(),
            })
            .collect::<Vec<_>>();

        return Ok(plugins);
    }
}