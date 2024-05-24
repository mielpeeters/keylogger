use crate::codes::Keys;
use crate::{cli, encrypt, files};
use miniz_oxide::deflate::compress_to_vec;
use miniz_oxide::inflate::decompress_to_vec_with_limit;
use serde::{Deserialize, Serialize};
#[cfg(feature = "bell")]
use soloud::{audio, AudioExt as _, LoadExt, Soloud};
use std::fmt::Debug;
use std::io::Read;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::{fs::OpenOptions, io::Write};

#[cfg(feature = "bell")]
static SOUND: &[u8] = include_bytes!("../bell.wav");

#[derive(Debug, Serialize, Deserialize)]
pub struct KeyLog(pub Vec<KeyPress>);

impl KeyLog {
    pub fn new() -> Self {
        KeyLog(Vec::new())
    }

    pub fn add(&mut self, key_press: KeyPress) {
        self.0.push(key_press);
    }

    fn log(&mut self, input_event: &[u8; 24]) {
        let key_press = KeyPress::new(input_event);
        self.add(key_press);
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    pub fn from_file<P>(path: P, password: &str) -> Option<Self>
    where
        P: AsRef<std::path::Path>,
    {
        let Ok(mut file) = OpenOptions::new().read(true).open(path) else {
            return None;
        };
        let mut encrypted = Vec::new();
        file.read_to_end(&mut encrypted).unwrap();
        let decrypted = encrypt::decrypt(&encrypted, password.as_bytes()).unwrap();
        let decompressed =
            decompress_to_vec_with_limit(&decrypted, 1 << 30).expect("Failed to decompress");
        Some(bincode::deserialize(&decompressed).unwrap())
    }

    pub fn to_file<P>(&self, path: P, password: &str)
    where
        P: AsRef<std::path::Path> + Debug,
    {
        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(&path)
            .unwrap_or_else(|_| {
                panic!("path {:?} could not be opened for truncated writing", path)
            });
        let encoded: Vec<u8> = bincode::serialize(&self).unwrap();
        let compressed = compress_to_vec(&encoded, 6);
        let encrypted = encrypt::encrypt(&compressed, password.as_bytes()).unwrap();
        file.write_all(&encrypted).unwrap();
    }
}

/// Represents the pressing of one key at some time
///
/// only records the down-stroke of a key, not holding or lifting
#[derive(Debug, Serialize, Deserialize)]
pub struct KeyPress {
    /// a timestamp
    pub time: SystemTime,
    /// a C short
    pub key: u16,
}

impl KeyPress {
    fn new(input_event: &[u8; 24]) -> Self {
        let key: [u8; 2] = input_event[18..20].try_into().unwrap();
        let key = u16::from_ne_bytes(key);
        let tv_sec: [u8; 8] = input_event[0..8].try_into().unwrap();
        let tv_sec = u64::from_ne_bytes(tv_sec);
        let tv_usec: [u8; 8] = input_event[8..16].try_into().unwrap();
        let tv_usec = u64::from_ne_bytes(tv_usec);
        let time = UNIX_EPOCH + Duration::from_secs(tv_sec) + Duration::from_micros(tv_usec);
        KeyPress { time, key }
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
        Some(path) => path.into(),
        None => files::log_bin()?,
    };
    // keyd virtual keyboard
    let event_path = format!("/dev/input/event{}", args.event);
    let mut file = OpenOptions::new().read(true).open(event_path)?;

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

            log.log(&buffer);
        }

        if term.load(Ordering::Relaxed) {
            log.to_file(&out_path, &password);
            break;
        }

        // backup to the file every 1000 key presses
        if log.len() != 0 && log.len() % 1000 == 0 {
            log.to_file(&out_path, &password);
        }
    }

    Ok(())
}

pub fn export(args: cli::ExportArgs) -> Result<(), Box<dyn std::error::Error>> {
    let log_path = match args.in_path {
        Some(path) => path.into(),
        None => files::log_bin()?,
    };
    let out_path = match args.out_path {
        Some(path) => path.into(),
        None => files::log_csv()?,
    };

    let password = rpassword::prompt_password("Password: ").unwrap();

    let log = KeyLog::from_file(log_path, &password).unwrap();
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(out_path)?;

    writeln!(file, "time,key")?;

    for key_press in log.0 {
        let key: Keys = unsafe { std::mem::transmute(key_press.key) };
        writeln!(file, "{:?},{}", key_press.time, key)?;
    }

    Ok(())
}

pub fn compress(args: &cli::CompressArgs) -> Result<(), Box<dyn std::error::Error>> {
    let log_path = match &args.in_path {
        Some(path) => path.into(),
        None => files::log_bin()?,
    };
    let out_path = match &args.out_path {
        Some(path) => path.into(),
        None => files::log_bin()?,
    };

    let password = rpassword::prompt_password("Password: ").unwrap();

    let mut file = OpenOptions::new().read(true).open(log_path).unwrap();
    let mut encrypted = Vec::new();
    file.read_to_end(&mut encrypted).unwrap();
    let decrypted = encrypt::decrypt(&encrypted, password.as_bytes()).unwrap();
    let log: KeyLog = bincode::deserialize(&decrypted).unwrap();

    log.to_file(out_path, &password);

    Ok(())
}
