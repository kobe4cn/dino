use std::{fs, path::Path};

use clap::Parser;
use dialoguer::Input;
use git2::Repository;

use crate::CmdExcetor;
use anyhow::Result;
use askama::Template;

//using askama crate to render templates
#[derive(Template)]
#[template(path = "config.yml.j2")]
struct ConfigYml<'a> {
    name: &'a str,
}

#[derive(Template)]
#[template(path = "main.ts.j2")]
struct MainTsFile {}

#[derive(Template)]
#[template(path = ".gitignore.j2")]
struct GitIgnoreFile {}
//askama template

#[derive(Debug, Parser)]
pub struct InitOpts {}

impl CmdExcetor for InitOpts {
    async fn execute(self) -> anyhow::Result<()> {
        //using dialoguer crate to get input from user
        let name: String = Input::new().with_prompt("Project name").interact_text()?;
        let cur = Path::new(".");
        //if current dir is empty then init the project, otherwise create new dir and init project
        if std::fs::read_dir(".")?.next().is_none() {
            init_project(&name, cur)?;
        } else {
            std::fs::create_dir(&name)?;
            let path = cur.join(&name);
            init_project(&name, &path)?;
        }
        Ok(())
    }
}

fn init_project(name: &str, path: &Path) -> Result<()> {
    //using git2 crate to init git repository
    Repository::init(path)?;
    //using askama crate to render templates
    let config = ConfigYml { name };
    fs::write(path.join("config.yml"), config.render()?)?;
    fs::write(path.join("main.ts"), MainTsFile {}.render()?)?;
    fs::write(path.join(".gitignore"), GitIgnoreFile {}.render()?)?;
    Ok(())
}
