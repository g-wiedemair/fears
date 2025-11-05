use crate::error::Error;
use std::path::PathBuf;

pub struct Project {
    name: PathBuf,
}

impl Project {
    pub fn new() -> Result<Project, Error> {
        let cwd = std::env::current_dir()?.join("project.dir");
        Ok(Project {
            name: cwd
        })
    }
}
