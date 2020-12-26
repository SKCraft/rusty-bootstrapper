#![feature(try_trait)]

use std::fmt;
use std::fs::{DirBuilder, File};
use std::ops::Try;
use std::option::NoneError;
use std::path::{Path, PathBuf};

use launch::LauncherBinary;

use crate::models::UpdateMeta;

mod launch;
mod version_check;
mod models;

trait Also: Sized {
    fn also<C>(mut self, call: C) -> Self where C: Fn(&mut Self) {
        call(&mut self);
        self
    }
}

impl<T: Sized> Also for T {}

struct BootstrapSettings {
    main_class: String,
    update_url: String,
}

struct Bootstrapper {
    base_dir: PathBuf,
    binaries_dir: PathBuf,
    portable: bool,
    bootstrap_args: Vec<String>,
    settings: BootstrapSettings,
}

#[derive(Debug)]
enum LaunchError {
    IoError(std::io::Error),
    MissingBinaries(NoneError),
    FailedDownload(reqwest::Error),
    LauncherExit(launch::JavaError),
}

impl fmt::Display for LaunchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LaunchError::IoError(e) =>
                write!(f, "IO error: {}", e),
            LaunchError::MissingBinaries(_) =>
                write!(f, "No launcher binaries found."),
            LaunchError::FailedDownload(e) =>
                write!(f, "Failed to download new launcher: {}", e),
            LaunchError::LauncherExit(e) =>
                write!(f, "Executing launcher: {}", e)
        }
    }
}

impl From<std::io::Error> for LaunchError {
    fn from(err: std::io::Error) -> Self {
        LaunchError::IoError(err)
    }
}

impl From<reqwest::Error> for LaunchError {
    fn from(err: reqwest::Error) -> Self {
        LaunchError::FailedDownload(err)
    }
}

impl Bootstrapper {
    fn base_dir(&self) -> &Path {
        &self.base_dir
    }

    fn binaries_dir(&self) -> &Path {
        &self.binaries_dir
    }

    fn run(&self) {
        if !self.binaries_dir().exists() {
            DirBuilder::new()
                .recursive(true)
                .create(self.binaries_dir())
                .expect("Could not create binaries dir.");
        }

        if !self.cleanup() {
            // TODO Show message box
            eprintln!("Failed cleaning up!");
            return
        }

        if let Err(err) = self.launch() {
            eprintln!("{}", err);
        }
    }

    fn cleanup(&self) -> bool {
        std::fs::read_dir(self.binaries_dir()).map(|dir| {
            dir.for_each(|r| {
                let entry = r.unwrap();

                if entry.path().extension()
                    .map(|ext| ext == "tmp")
                    .unwrap_or(false) {
                    if let Err(err) = std::fs::remove_file(entry.path()) {
                        eprintln!("Error deleting temporary file {:?}: {}", entry.path(), err);
                    }
                }
            });
        }).is_ok()
    }

    fn launch(&self) -> Result<(), LaunchError> {
        std::fs::read_dir(self.binaries_dir()).map_err(|e| {
            LaunchError::IoError(e)
        }).and_then(|dir| {
            let binaries: Vec<LauncherBinary> = dir.map(|r| {
                let entry = r.unwrap();

                LauncherBinary::new(entry.path())
            }).collect();

            if binaries.is_empty() {
                let new_binaries = self.download()?;
                self.launch_existing(new_binaries)
            } else {
                self.launch_existing(binaries)
            }
        })
    }

    fn download(&self) -> Result<Vec<LauncherBinary>, LaunchError> {
        let res: UpdateMeta = reqwest::blocking::get(&self.settings.update_url)?.json()?;
        let mut src = reqwest::blocking::get(&res.url)?;

        let mut filepath = self.binaries_dir.clone();
        filepath.push(format!("{}.tmp", &res.version));

        let mut dest = File::open(&filepath)?;
        std::io::copy(&mut src, &mut dest)?;
        std::fs::rename(&filepath, filepath.with_extension("jar"))?;

        let binary = LauncherBinary::new(filepath.clone());

        Ok(vec!(binary))
    }

    fn launch_existing(&self, binaries: Vec<LauncherBinary>) -> Result<(), LaunchError> {
        let working = binaries.iter().find(|bin| {
            println!("Trying {:?}...", bin.path());

            match bin.test_jar(&self.settings.main_class) {
                Ok(success) => success,
                Err(err) => {
                    eprintln!("Error reading JAR {:?}: {:?}", bin.path(), err);
                    false
                },
            }
        }).into_result().map_err(|e| LaunchError::MissingBinaries(e))?;

        binaries.iter()
            .filter(|b| &working != b)
            .for_each(|b| {
                b.delete();
        });

        let mut args = Vec::new();

        if let Some(base_dir) = self.base_dir().to_str() {
            args.push("--dir");
            args.push(base_dir);
        }

        if self.portable {
            args.push("--portable");
        }

        args.push("--bootstrap-version");
        args.push("1");

        for arg in self.bootstrap_args.iter() {
            args.push(arg);
        }

        let launcher = working.create_launcher(self.settings.main_class.clone(), args);
        launcher.launch().map_err(|e| LaunchError::LauncherExit(e))
    }
}

fn main() {
    let portable = Path::new("portable.txt").exists();
    let base_dir = match portable {
        true => Path::new(".").to_owned(),
        false => std::env::home_dir().expect("No home directory!")
            .also(|p: &mut PathBuf| p.push(".examplelauncher")),
    };

    let bootstrapper = Bootstrapper {
        portable,
        base_dir: base_dir.clone(),
        binaries_dir: base_dir.clone()
            .also(|p: &mut PathBuf| p.push("launcher")),
        bootstrap_args: std::env::args().skip(1).collect(),
        settings: BootstrapSettings {
            // TODO Pull these settings from a file
            main_class: String::from("com.skcraft.launcher.Launcher"),
            update_url: String::from("http://localhost:5000/latest.json")
        }
    };

    bootstrapper.run();
}
