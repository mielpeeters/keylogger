use dirs::data_dir;
use std::fs::{File, OpenOptions};
use std::path::PathBuf;

use crate::cli::{EncryptArgs, InitArgs};

pub fn init(args: InitArgs) -> Result<(), Box<dyn std::error::Error>> {
    let mut asset_path: PathBuf = args.path.into();
    asset_path.push("assets");

    let Some(mut data_dir) = data_dir() else {
        return Err("Could not find home directory".into());
    };
    data_dir.push("keylogger");

    // create the data directory if needed
    std::fs::create_dir_all(data_dir.join("assets"))?;

    let file = "qwerty_ansi_anno.svg";
    let mut src = File::open(asset_path.join(file))?;
    let output = data_dir.join("assets").join(file);
    let mut dst = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(output)?;
    std::io::copy(&mut src, &mut dst)?;

    Ok(())
}

pub fn log_bin() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let mut path = data_dir().ok_or("Could not find home directory")?;
    path.push("keylogger");
    path.push("assets");
    path.push("keylog.bin");

    Ok(path)
}

pub fn log_csv() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let mut path = data_dir().ok_or("Could not find home directory")?;
    path.push("keylogger");
    path.push("assets");
    path.push("keylog.csv");

    Ok(path)
}

pub fn qwerty_ansi() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let mut path = data_dir().ok_or("Could not find home directory")?;
    path.push("keylogger");
    path.push("assets");
    path.push("qwerty_ansi_anno.svg");

    Ok(path)
}

pub fn encrypt(args: EncryptArgs) -> Result<(), Box<dyn std::error::Error>> {
    let log_path = match args.in_path {
        Some(path) => path.into(),
        None => log_bin()?,
    };
    let out_path = match args.out_path {
        Some(path) => path.into(),
        None => log_bin()?,
    };
    let data = std::fs::read(log_path)?;
    let password = rpassword::prompt_password("Password: ").unwrap();
    let encrypted = crate::encrypt::encrypt(&data, password.as_bytes())?;
    std::fs::write(out_path, encrypted)?;

    Ok(())
}
