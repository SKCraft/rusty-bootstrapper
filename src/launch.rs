use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::Command;

use thiserror::Error;
use zip::result::ZipResult;
use zip::ZipArchive;

#[derive(Eq, PartialEq)]
pub struct LauncherBinary {
    path: PathBuf,
}

impl LauncherBinary {
    pub fn new(path: PathBuf) -> LauncherBinary {
        LauncherBinary { path }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn test_jar(&self) -> ZipResult<bool> {
        let jar_file = File::open(self.path())?;
        let zip = ZipArchive::new(jar_file)?;

        Ok(!zip.is_empty())
    }

    pub fn delete(&self) {
        std::fs::remove_file(self.path()).expect("Failed to remove a launcher file.");
    }

    pub fn create_launcher(&self, args: Vec<&str>) -> JavaLauncher {
        JavaLauncher {
            args: args.into_iter().map(String::from).collect(),
            opts: vec!(),
            jar_path: self.path.clone(),
        }
    }
}

#[derive(Error, Debug)]
pub enum JavaError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Launcher exited with non-zero status code {0}")]
    ExitCode(i32),
    #[error("Launcher was terminated by a signal")]
    Signal,
}

pub struct JavaLauncher {
    opts: Vec<String>,
    args: Vec<String>,
    jar_path: PathBuf,
}

impl JavaLauncher {
    pub fn launch(&self) -> Result<(), JavaError> {
        let cmd = Command::new("java")
            .args(&self.opts)
            .arg("-jar").arg(&self.jar_path)
            .args(&self.args)
            .status()?;

        if cmd.success() {
            Ok(())
        } else {
            match cmd.code() {
                Some(code) => Err(JavaError::ExitCode(code)),
                None => Err(JavaError::Signal),
            }
        }
    }
}
