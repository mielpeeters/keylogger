use crate::codes::Keys;
use crate::{cli, encrypt, files};
use serde::{Deserialize, Serialize};
#[cfg(feature = "bell")]
use soloud::{audio, AudioExt as _, LoadExt, Soloud};
use std::io::Read;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{fs::OpenOptions, io::Write};

#[cfg(feature = "bell")]
static SOUND: &[u8] = include_bytes!("../bell.wav");

#[derive(Debug, Serialize, Deserialize)]
pub struct KeyLog(pub Vec<KeyPress>);

impl KeyLog {
    fn new() -> Self {
        KeyLog(Vec::new())
    }

    fn add(&mut self, key_press: KeyPress) {
        self.0.push(key_press);
    }

    fn log(&mut self, key: u16) {
        let key_press = KeyPress::new(key);
        self.add(key_press);
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    pub fn from_file(path: &str, password: &str) -> Option<Self> {
        let Ok(mut file) = OpenOptions::new().read(true).open(path) else {
            return None;
        };
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).unwrap();
        let buffer = encrypt::decrypt(&buffer, password.as_bytes()).unwrap();
        Some(bincode::deserialize(&buffer).unwrap())
    }

    fn to_file(&self, path: &str, password: &str) {
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(path)
            .unwrap();
        let encoded: Vec<u8> = bincode::serialize(&self).unwrap();
        let encrypted = encrypt::encrypt(&encoded, password.as_bytes()).unwrap();
        file.write_all(&encrypted).unwrap();
    }
}

/// Represents the pressing of one key at some time
///
/// only records the down-stroke of a key, not holding or lifting
#[derive(Debug, Serialize, Deserialize)]
pub struct KeyPress {
    /// a timestamp
    pub time: u128,
    /// a C short
    pub key: u16,
}

impl KeyPress {
    fn new(key: u16) -> Self {
        KeyPress {
            time: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis(),
            key,
        }
    }
}

#[cfg(feature = "bell")]
fn check_and_bell(last_key: u16, key: u16) {
    if key == 14 && last_key != 42 && last_key != 14 {
        println!("Bell\x07");
        let sl = Soloud::default().unwrap();
        let mut wav = audio::Wav::default();
        wav.load_mem(SOUND).unwrap();
        sl.play(&wav);
        while sl.voice_count() > 0 {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }
}

pub fn log_keys(
    term: Arc<AtomicBool>,
    args: cli::LogArgs,
) -> Result<(), Box<dyn std::error::Error>> {
    let out_path = match args.out_path {
        Some(path) => path,
        None => {
            let path = files::log_bin()?;
            path.to_str().unwrap().to_string()
        }
    };
    // keyd virtual keyboard
    let path = format!("/dev/input/event{}", args.event);
    let mut file = OpenOptions::new().read(true).open(path)?;

    let mut buffer = [0; 24];

    // get the password from the user (blind typing)
    let password = rpassword::prompt_password("Password: ").unwrap();

    let mut log = match KeyLog::from_file(&out_path, &password) {
        Some(log) => log,
        None => KeyLog::new(),
    };

    #[cfg(feature = "bell")]
    let mut last_key = 0;

    loop {
        file.read_exact(&mut buffer)?;
        let value = buffer[20];
        let key: [u8; 2] = buffer[18..20].try_into()?;
        let key = u16::from_ne_bytes(key);
        if key != 0 && value == 1 {
            println!("{}: {key}", log.len());

            #[cfg(feature = "bell")]
            if args.bell {
                check_and_bell(last_key, key);
                last_key = key;
            }

            log.log(key);
        }

        if term.load(Ordering::Relaxed) {
            log.to_file(&out_path, &password);
            break;
        }

        if log.len() % 1000 == 0 {
            log.to_file(&out_path, &password);
        }
    }

    Ok(())
}

pub fn export(args: cli::ExportArgs) -> Result<(), Box<dyn std::error::Error>> {
    let log_path = match args.in_path {
        Some(path) => path,
        None => {
            let path = files::log_bin()?;
            path.to_str().unwrap().to_string()
        }
    };
    let out_path = match args.out_path {
        Some(path) => path,
        None => {
            let path = files::log_csv()?;
            path.to_str().unwrap().to_string()
        }
    };

    let password = rpassword::prompt_password("Password: ").unwrap();

    let log = KeyLog::from_file(&log_path, &password).unwrap();
    let mut file = OpenOptions::new().write(true).create(true).open(out_path)?;

    writeln!(file, "time,key")?;

    for key_press in log.0 {
        let key: Keys = unsafe { std::mem::transmute(key_press.key) };
        writeln!(file, "{},{}", key_press.time, key)?;
    }

    Ok(())
}
