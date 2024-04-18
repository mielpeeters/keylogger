/*!
* Drawing a heatmap in an svg image to show keyboard usage efficiency
*
* This module solves the following tasks:
* 1. Convert keypress file to a hashmap of keypress counts per key
* 2. Map keypress counts to a color gradient
* 3. Fill the annotated svg image with the color
*/

use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::{Read, Write};

use crate::cli::HeatmapArgs;
use crate::codes::Keys;
use crate::files;
use crate::keylog::KeyLog;

fn count_map(log: &KeyLog) -> HashMap<u16, u32> {
    let mut map = HashMap::new();
    for key_press in &log.0 {
        let key = key_press.key;
        let count = map.entry(key).or_insert(0);
        *count += 1;
    }
    map
}

#[allow(unused)]
fn red_heat(fraction: f32) -> u32 {
    let r = 0xff0000;
    let mut g = (255.0 * (1.0 - fraction)) as u32;
    let mut b = g;
    g = (g << 8) & 0xff00;
    b &= 0xff;

    r | g | b
}

/// Convert a fraction to a color in the form of a hex string
// TODO: use a perceptually uniform color scale
fn heat(fraction: f32) -> String {
    let gradient = colorous::ORANGE_RED;
    format!("{:x}", gradient.eval_continuous(fraction.into()))
}

/// Parse the annotated svg
fn fill_heat_in_kbd(
    svg: &mut String,
    key: u16,
    heat: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let key: Keys = unsafe { std::mem::transmute(key) };
    let pattern = format!("{{{{{}}}", key);
    let Some(start) = svg.find(&pattern) else {
        return Ok(());
    };
    let end = start + pattern.len();

    svg.replace_range(start..=end, &heat);

    Ok(())
}

fn fill_empty(svg: &mut String) -> Result<(), Box<dyn std::error::Error>> {
    let pattern = "#{{";
    let Some(start) = svg.find(pattern) else {
        return Err("Could not find empty key".into());
    };
    let end = svg[start..].find('}').unwrap();
    let end = start + end + 1;

    svg.replace_range(start..=end, "#ffffff");

    Ok(())
}

pub fn heatmap(args: HeatmapArgs) -> Result<(), Box<dyn std::error::Error>> {
    let log_path = match args.in_path {
        Some(path) => path.into(),
        None => files::log_bin()?,
    };
    let kbd_svg_path = match args.keyboard_svg_path {
        Some(path) => path.into(),
        None => files::qwerty_ansi()?,
    };

    let password = rpassword::prompt_password("Password: ").unwrap();

    let log = KeyLog::from_file(log_path, &password).unwrap();

    let mut keyboard_svg = OpenOptions::new().read(true).open(kbd_svg_path)?;
    let mut kbd = String::new();
    keyboard_svg.read_to_string(&mut kbd)?;

    let mut output_svg = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&args.out_path)?;

    let counts = count_map(&log);
    let max = counts.values().max().unwrap();

    for (key, count) in &counts {
        let fraction = *count as f32 / *max as f32;
        let heat = heat(fraction);
        fill_heat_in_kbd(&mut kbd, *key, heat)?;
    }

    while fill_empty(&mut kbd).is_ok() {}

    output_svg.write_all(kbd.as_bytes())?;

    Ok(())
}
