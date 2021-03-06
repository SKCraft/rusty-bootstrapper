use std::cmp::Ordering;
use std::convert::{TryFrom, TryInto};
use std::fmt;
use std::num::ParseIntError;
use std::ops::Try;
use std::process::Command;
use std::string::FromUtf8Error;

use regex::bytes::Regex;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CommandError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Invalid (non-UTF8) output from Java command")]
    InvalidEncoding(#[from] FromUtf8Error),
    #[error("Invalid output from Java command")]
    InvalidOutput,
    #[error("{0}")]
    InvalidVersion(VersionFormatError),
}

#[derive(Error, Debug)]
pub enum VersionFormatError {
    #[error("Not enough parts in version (should be 3)")]
    NotEnoughParts,
    #[error("Too many parts in version (should be 3)")]
    TooManyParts,
    #[error("Invalid number in major or minor version: {0}")]
    InvalidNumber(#[from] ParseIntError),
}

struct JavaVersion {
    major: u16,
    minor: u16,
    build: String,
}

impl Ord for JavaVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        self.major.cmp(&other.major)
            .then(self.minor.cmp(&other.minor))
    }
}

impl PartialOrd for JavaVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for JavaVersion {
    fn eq(&self, other: &Self) -> bool {
        self.major == other.major && self.minor == other.minor
    }
}

impl Eq for JavaVersion {}

impl fmt::Display for JavaVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{}.{}", self.major, self.minor))
    }
}

impl TryFrom<String> for JavaVersion {
    type Error = VersionFormatError;

    fn try_from(version_str: String) -> Result<Self, Self::Error> {
        let parts: Vec<&str> = version_str.split('.').collect();

        match parts.len() {
            x if x < 3 => Err(VersionFormatError::NotEnoughParts),
            x if x > 3 => Err(VersionFormatError::TooManyParts),
            _ => {
                Ok(JavaVersion {
                    major: parts[0].parse()?,
                    minor: parts[1].parse()?,
                    build: parts[2].into(),
                })
            }
        }
    }
}

impl JavaVersion {
    pub fn new(major: u16, minor: u16) -> JavaVersion {
        JavaVersion {
            major, minor,
            build: "0".into(),
        }
    }

    pub fn system_java_version() -> Result<JavaVersion, CommandError> {
        let output = Command::new("java").arg("-version").output()?;

        let pattern = Regex::new(r#"version\s"([0-9._-]+)""#).unwrap();
        match pattern.captures(&output.stderr) {
            None => Err(CommandError::InvalidOutput),
            Some(captures) => {
                captures.get(1).into_result()
                    .map_err(|_| CommandError::InvalidOutput)
                    .and_then(|m| {
                        String::from_utf8(m.as_bytes().into())
                            .map_err(CommandError::InvalidEncoding)
                    }).and_then(|version_str| {
                        version_str.try_into()
                            .map_err(CommandError::InvalidVersion)
                    })
            }
        }
    }
}

pub fn check_java_version() -> Result<bool, CommandError> {
    let desired = JavaVersion::new(1, 8);
    let available = JavaVersion::system_java_version()?;

    eprintln!("Found Java version {}", available);

    return Ok(available >= desired)
}

