use std::cmp::Ordering;
use std::convert::{TryFrom, TryInto};
use std::num::ParseIntError;
use std::ops::Try;
use std::process::Command;
use std::string::FromUtf8Error;

use regex::bytes::Regex;

#[derive(Debug)]
enum CommandError {
    IoError(std::io::Error),
    InvalidOutput,
    InvalidVersion(VersionFormatError),
}

impl From<std::io::Error> for CommandError {
    fn from(err: std::io::Error) -> Self {
        CommandError::IoError(err)
    }
}

impl From<FromUtf8Error> for CommandError {
    fn from(_: FromUtf8Error) -> Self {
        CommandError::InvalidOutput
    }
}

#[derive(Debug)]
enum VersionFormatError {
    NotEnoughParts,
    TooManyParts,
    InvalidNumber(ParseIntError),
}

impl From<ParseIntError> for VersionFormatError {
    fn from(err: ParseIntError) -> Self {
        VersionFormatError::InvalidNumber(err)
    }
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

        let pattern = Regex::new(r#"version\s"(\d+)""#).unwrap();
        match pattern.captures(&output.stderr) {
            None => Err(CommandError::InvalidOutput),
            Some(captures) => {
                captures.get(1).into_result()
                    .map_err(|_| CommandError::InvalidOutput)
                    .and_then(|m| {
                        String::from_utf8(m.as_bytes().into())
                            .map_err(|e| e.into())
                    }).and_then(|version_str| {
                        version_str.try_into()
                            .map_err(CommandError::InvalidVersion)
                    })
            }
        }
    }
}

