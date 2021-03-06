#![feature(try_trait)]

use std::fs::{DirBuilder, File};
use std::ops::Try;
use std::option::NoneError;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use thiserror::Error;

use launch::LauncherBinary;
use models::UpdateMeta;
use platform::HomeDirError;
use self_reader::ReadError;

mod launch;
mod version_check;
mod models;
mod ui;
mod self_reader;
mod platform;

#[derive(Deserialize)]
struct BootstrapSettings {
    update_url: String,
    home_dir: String,
    home_dir_windows: String,
}

struct Bootstrapper {
    base_dir: PathBuf,
    binaries_dir: PathBuf,
    portable: bool,
    bootstrap_args: Vec<String>,
    settings: BootstrapSettings,
}

#[derive(Error, Debug)]
enum LaunchError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("No launcher binaries found.")]
    MissingBinaries(NoneError),
    #[error("Failed to download new launcher: {0}")]
    FailedDownload(#[from] reqwest::Error),
    #[error("Launcher exited unexpectedly: {0}")]
    LauncherExit(#[from] launch::JavaError),
    #[error("Invalid Java version: {0}")]
    InvalidJava(String),
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
            let err = DirBuilder::new()
                .recursive(true)
                .create(self.binaries_dir())
                .err();

            if err.is_some() {
                ui::show_dialog("Failed to create binaries dir.");
                return
            }
        }

        if !self.cleanup() {
            ui::show_dialog("Failed cleaning up!");
            return
        }

        if let Err(err) = self.launch() {
            ui::show_dialog(&format!("{}", err));
        }
    }

    fn cleanup(&self) -> bool {
        std::fs::read_dir(self.binaries_dir()).map(|dir| {
            dir.for_each(|r| {
                let entry = r.unwrap();

                if entry.path().extension()
                    .map_or(false, |ext| ext == "tmp")
                {
                    if let Err(err) = std::fs::remove_file(entry.path()) {
                        eprintln!("Error deleting temporary file {:?}: {}", entry.path(), err);
                    }
                }
            });
        }).is_ok()
    }

    fn launch(&self) -> Result<(), LaunchError> {
        match version_check::check_java_version() {
            Ok(valid) => {
                if !valid {
                    // TODO Report the Java version that was incorrect
                    return Err(LaunchError::InvalidJava("Need at least Java 8".into()))
                }
            }
            Err(err) => {
                eprintln!("Couldn't check Java version: {}", err);
                // just continue, failed to check version.
                // if we fail later that will be reported.
            }
        }

        std::fs::read_dir(self.binaries_dir()).map_err(|e| {
            LaunchError::IoError(e)
        }).and_then(|dir| {
            let binaries: Vec<LauncherBinary> = dir.map(|r| {
                let entry = r.unwrap();

                LauncherBinary::new(entry.path())
            }).collect();

            if binaries.is_empty() {
                let new_binaries = self.download()?;
                self.launch_existing(&new_binaries)
            } else {
                self.launch_existing(&binaries)
            }
        })
    }

    fn download(&self) -> Result<Vec<LauncherBinary>, LaunchError> {
        let res: UpdateMeta = reqwest::blocking::get(&self.settings.update_url)?.json()?;
        let mut src = reqwest::blocking::get(&res.url)?;

        eprintln!("Downloading launcher version {} from \"{}\"", &res.version, &res.url);

        let filepath = self.binaries_dir().join(format!("{}.tmp", &res.version));

        let mut dest = File::create(&filepath)?;
        std::io::copy(&mut src, &mut dest)?;

        let target_name = filepath.with_extension("jar");
        std::fs::rename(&filepath, &target_name)?;

        eprintln!("Downloaded {:?}", target_name.file_name().unwrap());

        let binary = LauncherBinary::new(target_name);

        Ok(vec!(binary))
    }

    fn launch_existing(&self, binaries: &[LauncherBinary]) -> Result<(), LaunchError> {
        let working = binaries.iter().find(|bin| {
            eprintln!("Trying {:?}...", bin.path());

            match bin.test_jar() {
                Ok(success) => success,
                Err(err) => {
                    eprintln!("Error reading JAR {:?}: {:?}", bin.path(), err);
                    false
                },
            }
        }).into_result().map_err(LaunchError::MissingBinaries)?;

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

        for arg in &self.bootstrap_args {
            args.push(arg);
        }

        let launcher = working.create_launcher(args);
        launcher.launch().map_err(LaunchError::LauncherExit)
    }
}

#[derive(Error, Debug)]
enum BootstrapError {
    #[error("Embedded data error: {0}")]
    EmbeddedDataError(#[from] ReadError),
    #[error("Embedded data is corrupted!\n {0}")]
    EmbeddedDataCorrupt(#[from] serde_json::Error),
    #[error("Home directory is missing: {0}")]
    HomeDirMissing(#[from] HomeDirError),
}

fn startup() -> Result<(), BootstrapError> {
    let embedded_settings = self_reader::read_appended_data()?;
    let settings: BootstrapSettings = serde_json::from_str(&embedded_settings)?;

    let home_dir = if cfg!(windows) {
        &settings.home_dir_windows
    } else {
        &settings.home_dir
    };
    let portable = Path::new("portable.txt").exists();
    let base_dir = if portable {
        PathBuf::from(".")
    } else {
        platform::home_dir()?.join(home_dir)
    };

    eprintln!("Using base dir {:?}", base_dir);

    let bootstrapper = Bootstrapper {
        portable,
        binaries_dir: base_dir.join("launcher"),
        base_dir,
        bootstrap_args: std::env::args().skip(1).collect(),
        settings,
    };

    bootstrapper.run();
    Ok(())
}

fn main() {
    if let Err(err) = startup() {
        ui::show_dialog(&format!("{}", err));
    }
}
