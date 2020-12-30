use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::string::FromUtf8Error;

use byteorder::{BigEndian, ReadBytesExt};

#[derive(Debug)]
pub enum ReadError {
    IoError(std::io::Error),
    EncodingError(FromUtf8Error),
}

impl From<std::io::Error> for ReadError {
    fn from(err: std::io::Error) -> Self {
        ReadError::IoError(err)
    }
}

pub fn read_appended_data() -> Result<String, ReadError> {
    let exe = std::env::current_exe()?;
    let mut file = File::open(exe)?;

    let footer_pos = file.seek(SeekFrom::End(-8))?;
    let data_pos = file.read_u64::<BigEndian>()?;
    file.seek(SeekFrom::Start(data_pos))?;

    let mut result_buf = Vec::new();
    result_buf.resize((footer_pos - data_pos) as usize, 0);
    file.read_exact(&mut result_buf)?;

    let result = String::from_utf8(result_buf)
        .map_err(ReadError::EncodingError)?;
    Ok(result)
}
