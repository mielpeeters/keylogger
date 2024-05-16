/*!
* Analyze keylog data to find which words are most commonly typed.
*/

use std::collections::HashMap;

use crate::{cli::WordsArgs, codes::Keys, files, keylog::KeyLog};

pub fn parse_words(log: &KeyLog) -> Result<HashMap<String, u64>, Box<dyn std::error::Error>> {
    let mut words = HashMap::new();

    let written = log.0.iter().map(|press| {
        let key: Keys = unsafe { std::mem::transmute(press.key) };
        key.written()
    });

    // println!("{:?}", written.clone().collect::<Vec<_>>());

    let mut word_list = vec![];
    let mut current_word = String::new();

    // TODO: only add letters to each other if they are pressed in quick succession
    written.for_each(|char| {
        if let Some(c) = char {
            current_word.push(c);
        } else if !current_word.is_empty() {
            word_list.push(current_word.clone());
            current_word.clear();
        }
    });

    word_list.retain(|w| w.len() >= 2);

    for word in word_list {
        let count = words.entry(word).or_insert(0);
        *count += 1;
    }

    Ok(words)
}

pub fn words(args: &WordsArgs) -> Result<(), Box<dyn std::error::Error>> {
    let log_path = match &args.in_path {
        Some(path) => path.into(),
        None => files::log_bin()?,
    };

    // get the password from the user (blind typing)
    let password = rpassword::prompt_password("Password: ").unwrap();

    let log = match KeyLog::from_file(&log_path, &password) {
        Some(log) => log,
        None => return Err(format!("Keylog file {:?} does not exist...", log_path).into()),
    };

    let words = parse_words(&log)?;

    // pretty print the results, sorted
    let mut words: Vec<_> = words.into_iter().collect();
    words.sort_by(|a, b| b.1.cmp(&a.1));
    words.iter().take(100).for_each(|(word, count)| {
        println!("{:20} : {}", word, count);
    });

    Ok(())
}
