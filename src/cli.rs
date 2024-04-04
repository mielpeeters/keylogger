use clap::{Args, Parser, Subcommand};
use nestify::nest;

nest! {
    #[derive(Parser, Debug)]
    pub struct Cli {
        /// The subcommand to run
        #[clap(subcommand)]
        pub subcmd:
            #[derive(Subcommand, Debug)]
            pub enum SubCommand {
                /// Initialize files
                Init(InitArgs),
                /// Log all keypresses
                Log(LogArgs),
                /// Export log to csv
                Export(ExportArgs),
                /// Draw a heatmap of keypresses
                Heatmap(HeatmapArgs),
            },
    }
}

#[derive(Args, Debug)]
pub struct InitArgs {
    /// The path to the repository directory
    #[arg(short, long, default_value = ".")]
    pub path: String,
}

#[derive(Args, Debug)]
pub struct LogArgs {
    /// The path to the log file
    #[arg(short, long)]
    pub out_path: Option<String>,

    #[cfg(feature = "bell")]
    /// Sound a bell when backspace is pressed wrongly
    #[arg(short, long)]
    pub bell: bool,

    /// The events file number to log on
    #[arg(short, long, default_value_t = 21)]
    pub event: u32,
}

#[derive(Args, Debug)]
pub struct ExportArgs {
    /// The path to the log file
    #[arg(short, long)]
    pub in_path: Option<String>,

    /// The path to the csv file
    #[arg(short, long)]
    pub out_path: Option<String>,
}

#[derive(Args, Debug)]
pub struct HeatmapArgs {
    /// The path to the log file
    #[arg(short, long)]
    pub in_path: Option<String>,

    /// The path to the svg file
    #[arg(short, long)]
    pub keyboard_svg_path: Option<String>,

    /// The path to the output svg file
    #[arg(short, long, default_value = "heatmap.svg")]
    pub out_path: String,
}
