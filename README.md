# Keylogger

Log your keystrokes and generate heatmaps of your typing patterns.



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

First, find out which event file in `/dev/input/` is your keyboard. 
You can use the tool `evtest` to find the event file number (`eventX`).

```shell
sudo keylogger log -e X
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

# Changelog 
- `0.3.0` -> `0.3.1`: added `words` subcommand to show the top 100 most used words in your keylog data file.
- `0.2.0` -> `0.3.0`: added `analyze` subcommand to show when you used your computer. Also changed internal time format to `SystemTime`, and added `convert` subcommand to transform your keylog data file to the new format for backward compatibility.
- `0.1.0` -> `0.2.0`: added AES256-GCM password-protected encryption. Also added `encrypt` subcommand to encrypt `0.1.0` keylog files to ensure compatibility with `0.2.0`.

# Roadmap

- add different keyboard layouts support
