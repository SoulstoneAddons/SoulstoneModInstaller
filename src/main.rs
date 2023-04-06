mod steam;
mod installer;
mod config;
mod plugins;
mod github_repositories;
mod plugin;
mod github_releases;

use std::io::Read;
use ansi_term::Color::{Red, White, Green, Yellow};
use ansi_term::{Colour, enable_ansi_support};
use crate::config::*;
use crate::installer::Installer;
use crate::steam::Steam;


fn main() {
    // enable ansi support
    let _ = enable_ansi_support();

    // set title of console via ansi
    print!("\x1b]0;{} - v{}\x07", TITLE, VERSION);
    // hide cursor
    print!("\x1b[?25l");

    println!("{} v{}", TITLE, VERSION);
    println!("Author: {}", AUTHOR);
    println!("License: {}", LICENSE);
    println!("{}", Red.paint("Note: If you get rate-limited, please wait a few minutes and try again."));
    println!("{}", Red.paint("Unfortunately, this is a limitation of GitHub's API."));
    println!();

    begin_installation();

    // press any key to exit
    println!("{}", White.paint("Press any key to exit..."));
    std::io::stdin().read(&mut [0]).unwrap();
}

fn begin_installation() {
    let gray = Colour::RGB(128, 128, 128);

    println!("{}", gray.paint("Checking for Steam..."));
    let steam_path = Steam::get_steam_path();
    if steam_path.is_none() {
        println!("{}", Red.paint("Steam not found!"));
        return;
    }
    println!("{}", Green.paint("Steam found!"));
    let steam_path = steam_path.unwrap();
    println!("{}", gray.paint("Checking for games..."));
    let games = Steam::iterate_games(&steam_path);
    if games.is_none() {
        println!("{}", Red.paint("No games found!"));
        return;
    }
    println!("{}", Green.paint(format!("{} Games found!", games.as_ref().unwrap().len())));
    let games = games.unwrap();
    let soulstone = games.iter().find(|game| game.id == "2066020");
    if soulstone.is_none() {
        println!("{}", Red.paint("Soulstone Survivors not found!"));
        return;
    }
    println!("{}", Green.paint("Soulstone Survivors found!"));
    let soulstone_game = soulstone.unwrap();
    let installer = Installer::new(&soulstone_game.path);

    println!("{}", gray.paint("Installing BepInEx..."));

    let result = installer.install();
    // Install BepInEx
    if let Err(err) = result {
        println!("{}", Red.paint(format!("{}", err)));
        return;
    }
    let result = result.unwrap();
    if result.installed_bepinex.is_some() && result.installed_bepinex.unwrap() {
        println!("{}", Green.paint("BepInEx installed!"));
    } else {
        println!("{}", Yellow.paint("BepInEx already installed!"));
    }

    // Install Plugins
    if result.plugins.is_some() {
        let plugins = result.plugins.unwrap();
        println!("{}", Green.paint(format!("{} Plugins installed!", plugins.len())));
    } else {
        println!("{}", Green.paint("No plugins installed!"));
    }
}
