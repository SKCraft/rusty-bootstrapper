use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::string::FromUtf8Error;

use byteorder::{BigEndian, ReadBytesExt};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ReadError {
    #[error("Error reading file: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Invalid magic value; your bootstrapper is missing required data!")]
    InvalidMagic,
    #[error("Bootstrapper data is not valid UTF-8")]
    EncodingError(#[from] FromUtf8Error),
}

pub fn read_appended_data() -> Result<String, ReadError> {
    let exe = std::env::current_exe()?;
    let mut file = File::open(exe)?;

    let footer_pos = file.seek(SeekFrom::End(-12))?;
    let magic = file.read_u32::<BigEndian>()?;
    if magic != 0xDEADBEEF {
        return Err(ReadError::InvalidMagic)
    }

    let data_pos = file.read_u64::<BigEndian>()?;
    file.seek(SeekFrom::Start(data_pos))?;

    let mut result_buf = Vec::new();
    result_buf.resize((footer_pos - data_pos) as usize, 0);
    file.read_exact(&mut result_buf)?;

    let result = String::from_utf8(result_buf)?;
    Ok(result)
}
