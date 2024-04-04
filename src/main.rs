/*!
* Log your keystrokes and generate heatmaps of your typing patterns.
*
*
* **Beware**: the key log is not encrypted and contains all keystrokes (including passwords). Use at your own risk.
*
* # Installation
*
* ```shell
* git clone git@github.com/mielpeeters/keylogger
* cd keylogger
* cargo install --path .
* sudo keylogger init .
* ```
* This will generate a `keylogger` bin in your cargo bin directory.
* The `init` subcommand will move the required assets to `XDG_DATA_HOME/keylogger` such that the
* command can be used from within any directory.
*
* # Usage
*
* ```shell
* sudo keylogger log
* ```
* This will start the logging, with default output path (./assets/keylog.bin). Run with `-h` to see more options.
*
*
* You can output as a csv with the `export` subcommand.
*
*
* To generate the heatmap svg, run:
* ```shell
* sudo keylogger heatmap -o heatmap.svg
* ```
* This will generate a heatmap image at `heatmap.svg`.
* Log your keystrokes and generate heatmaps of your typing patterns.
*/

use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use clap::Parser;

mod cli;
mod codes;
mod encrypt;
mod files;
mod heatmap;
mod keylog;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let term = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&term))?;

    let args: cli::Cli = cli::Cli::parse();

    match args.subcmd {
        cli::SubCommand::Init(i) => files::init(i),
        cli::SubCommand::Log(l) => keylog::log_keys(Arc::clone(&term), l),
        cli::SubCommand::Export(e) => keylog::export(e),
        cli::SubCommand::Heatmap(h) => heatmap::heatmap(h),
        cli::SubCommand::Encrypt(e) => files::encrypt(e),
    }
}
