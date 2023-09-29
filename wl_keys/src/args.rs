/// The 'daemon' subcmd
#[derive(clap::Subcommand)]
pub enum DaemonCmd {
    /// Run the daemon
    Start,
    /// Stop the daemon if it's currently running
    Stop,
    /// List the protocols supported by the wm
    Protocols,
}

/// The 'auto' subcmd
#[derive(clap::Subcommand)]
pub enum AutoCmd {
    /// Enable the input detection
    Enable,
    /// Disable the input detection
    Disable,
    /// Toggle the input detection
    Toggle,
    /// Returns true if input detection is on
    Query,
}

/// The 'ui' subcmd
#[derive(clap::Subcommand)]
pub enum UiCmd {
    /// Show the ui
    Open,
    /// Hide the ui
    Close,
    /// Toggle the ui
    Toggle,
}

/// The 'mod' subcmd
#[derive(clap::Subcommand)]
pub enum ModCmd {
    /// Press a modifier
    Press {
        /// The modifier to press
        modifier: String,
    },
    /// Release a modifier
    Release {
        /// The modifier to release
        modifier: String,
    },
    /// Toggle a modifier
    Toggle {
        /// The modifier to toggle
        modifier: String,
    },
    /// Query a modifier state
    Query {
        /// The modifier to query
        modifier: String,
    },
}

/// The top level args
#[derive(clap::Parser)]
#[command(name = "wl_keys")]
pub enum Command {
    /// Manage the daemon
    #[command(subcommand)]
    Daemon(DaemonCmd),

    /// Manage the input detection
    #[command(subcommand)]
    Auto(AutoCmd),

    /// Manage the ui
    #[command(subcommand)]
    Ui(UiCmd),

    /// Press and release modifiers
    #[command(subcommand)]
    Mod(ModCmd),

    /// Press a key
    Key {
        /// The key to press
        key: String,
    },
}
