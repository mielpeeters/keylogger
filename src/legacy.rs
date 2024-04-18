/*!
* Support conversion between old keylog format and new keylog format.
*/

use std::{fs::OpenOptions, io::Read, path::Path};

use serde::{Deserialize, Serialize};

use crate::{cli::ConvertArgs, encrypt, files, keylog::KeyLog};

#[derive(Debug, Serialize, Deserialize)]
pub struct OldKeyPress {
    /// a timestamp: milliseconds since epoch
    pub time: u128,
    /// a C short
    pub key: u16,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OldKeyLog(pub Vec<OldKeyPress>);

impl OldKeyLog {
    pub fn from_file<P>(path: P, password: &str) -> Option<Self>
    where
        P: AsRef<Path>,
    {
        let Ok(mut file) = OpenOptions::new().read(true).open(path) else {
            return None;
        };
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).unwrap();
        let buffer = encrypt::decrypt(&buffer, password.as_bytes()).unwrap();
        Some(bincode::deserialize(&buffer).unwrap())
    }
}

pub fn convert(args: ConvertArgs) -> Result<(), Box<dyn std::error::Error>> {
    let old_path = match args.in_path {
        Some(path) => path.into(),
        None => files::log_bin()?,
    };
    let new_path = match args.out_path {
        Some(path) => path.into(),
        None => files::log_bin()?,
    };

    let password = rpassword::prompt_password("Password: ").unwrap();

    let old_log = OldKeyLog::from_file(old_path, &password).unwrap();

    let mut new_log = KeyLog::new();

    old_log.0.iter().for_each(|old_key_press| {
        let key_press = crate::keylog::KeyPress {
            time: std::time::UNIX_EPOCH
                + std::time::Duration::from_millis(old_key_press.time.try_into().unwrap()),
            key: old_key_press.key,
        };
        new_log.add(key_press);
    });

    new_log.to_file(new_path, &password);

    Ok(())
}
