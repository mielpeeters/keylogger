/*!
* Analyse the keylog to find how much time was spent typing (working) each day.
*/

use std::{
    fmt::Display,
    time::{SystemTime, UNIX_EPOCH},
};

use chrono::{DateTime, Utc};
use chrono_tz::{Europe::Brussels, Tz};

use crate::{cli::AnalyzeTimeArgs, files, keylog::KeyLog};

const PAUSE_MIN_MINUTES: u64 = 5;
const MIN_KEYS: u64 = 30;

#[derive(Clone, Debug)]
struct WorkSession {
    start: SystemTime,
    end: SystemTime,
    presses: u64,
}

impl Display for WorkSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let start_time = system_to_brussels_time(self.start);
        let end_time = system_to_brussels_time(self.end);
        let duration = self.end.duration_since(self.start).unwrap().as_secs() / 60;
        let hrs = duration / 60;
        let mins = duration % 60;
        write!(
            f,
            "\x1b[1m{} - {}\x1b[0m ({} keys @ {}h{}m)",
            start_time.format("%H:%M"),
            end_time.format("%H:%M"),
            self.presses,
            hrs,
            mins
        )
    }
}

impl WorkSession {
    fn secs(&self) -> u64 {
        self.end.duration_since(self.start).unwrap().as_secs()
    }
}

fn system_to_brussels_time(system_time: SystemTime) -> DateTime<Tz> {
    let time: DateTime<Utc> = DateTime::from(system_time);
    time.with_timezone(&Brussels)
}

pub fn analyze_time(args: AnalyzeTimeArgs) -> Result<(), Box<dyn std::error::Error>> {
    let log_path = match args.in_path {
        Some(path) => path.into(),
        None => files::log_bin()?,
    };

    // get the password from the user (blind typing)
    let password = rpassword::prompt_password("Password: ").unwrap();

    let log = match KeyLog::from_file(&log_path, &password) {
        Some(log) => log,
        None => return Err(format!("Keylog file {:?} does not exist...", log_path).into()),
    };

    // loop over all keystrokes and map into work sessions
    let mut sessions: Vec<WorkSession> = vec![];
    let mut current_session = WorkSession {
        start: log.0.get(0).unwrap().time,
        end: log.0.get(0).unwrap().time,
        presses: 1,
    };
    log.0.iter().skip(1).for_each(|key_press| {
        if key_press
            .time
            .duration_since(current_session.end)
            .unwrap()
            .as_secs()
            > PAUSE_MIN_MINUTES * 60
        {
            if current_session.presses > MIN_KEYS {
                // add this worksession to the list
                sessions.push(current_session.clone());
            }
            // start a new session
            current_session.start = key_press.time;
            current_session.end = key_press.time;
            current_session.presses = 0;
        } else {
            current_session.end = key_press.time;
            current_session.presses += 1;
        }
    });
    if current_session.presses > MIN_KEYS {
        // add this worksession to the list
        sessions.push(current_session.clone());
    }

    let mut current_datetime = system_to_brussels_time(UNIX_EPOCH);
    let mut total_work_secs = 0;

    // export sessions in some way (just print for now)
    sessions.iter().enumerate().for_each(|(i, session)| {
        let datetime = system_to_brussels_time(session.start);
        if datetime.date_naive() != current_datetime.date_naive() {
            if i > 0 {
                println!("Total: {:.2} hrs", total_work_secs as f32 / 3600.0);
                total_work_secs = 0;
            }
            println!("{}", datetime.format("%Y-%m-%d"));
            current_datetime = datetime;
        }
        println!("{}", session);
        total_work_secs += session.secs();
    });
    println!("Total: {:.2} hrs", total_work_secs as f32 / 3600.0);

    Ok(())
}
