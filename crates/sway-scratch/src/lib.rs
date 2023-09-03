use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// toggle named scratchpad
    Show {
        /// criteria
        #[command(flatten)]
        criteria: Criteria,

        /// the command to open the scratch initially
        #[arg(long)]
        exec: String,

        /// resize command to run when the scratch is shown (e.g. "set 90 ppt 90 ppt")
        #[arg(long)]
        resize: Option<String>,
    },
}

#[derive(Args)]
#[group(required = true, multiple = false)]
pub struct Criteria {
    /// the Wayland app_id of the application
    #[arg(long)]
    pub app_id: Option<String>,

    /// the window_properties.class of the application (Xwayland)
    #[arg(long)]
    pub class: Option<String>,
}
