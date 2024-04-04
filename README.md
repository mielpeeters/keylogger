# Keylogger

Log your keystrokes and generate heatmaps of your typing patterns.


**Beware**: the key log is not encrypted and contains all keystrokes (including passwords). Use at your own risk.

# Installation

```shell
git clone git@github.com/mielpeeters/keylogger
cd keylogger
cargo build --release
sudo ./target/release/keylogger init
```
The `init` subcommand will move the required assets to `XDG_DATA_HOME/keylogger` such that the 
command can be used from within any directory.

Note: this is the data directory of the superuser.

# Usage

```shell
sudo keylogger log
```
This will start the logging, with default output path (./assets/keylog.bin). Run with `-h` to see more options.


You can output as a csv with the `export` subcommand.


To generate the heatmap svg, run:
```shell
sudo keylogger heatmap -o heatmap.svg
```
This will generate a heatmap image at `heatmap.svg`.

# Features
The `bell` feature is something I implemented for myself, and won't work generally.


# Roadmap

- add different keyboard layouts support
- add encryption of the output file with user-supplied password
