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
                /// Encrypt the unencrypted log
                Encrypt(EncryptArgs),
                /// Analyse the keylog to find timing info.
                Analyze(AnalyzeTimeArgs),
                /// Convert to new keylog format
                Convert(ConvertArgs),
                /// Find which words are typed most often
                Words(WordsArgs),
                /// Compress the log file
                Compress(CompressArgs)
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
    #[arg(short, long, default_value_t = 12)]
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

#[derive(Args, Debug)]
pub struct EncryptArgs {
    /// The path to the log file
    #[arg(short, long)]
    pub in_path: Option<String>,

    /// The path to the encrypted output
    #[arg(short, long)]
    pub out_path: Option<String>,
}

#[derive(Args, Debug)]
pub struct AnalyzeTimeArgs {
    /// The path to the log file
    #[arg(short, long)]
    pub in_path: Option<String>,
}

#[derive(Args, Debug)]
pub struct ConvertArgs {
    /// The path to the old keylog file
    #[arg(short, long)]
    pub in_path: Option<String>,

    /// The path to the new keylog file
    #[arg(short, long)]
    pub out_path: Option<String>,
}

#[derive(Args, Debug)]
pub struct WordsArgs {
    /// The path to the log file
    #[arg(short, long)]
    pub in_path: Option<String>,

    /// Minimum word length
    #[arg(short, long, default_value = "2")]
    pub length: usize,
}

#[derive(Args, Debug)]
pub struct CompressArgs {
    /// The path to the log file
    #[arg(short, long)]
    pub in_path: Option<String>,

    /// The path to the compressed output
    #[arg(short, long)]
    pub out_path: Option<String>,
}
