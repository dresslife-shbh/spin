use anyhow::Error;
use clap::{CommandFactory, Parser, Subcommand};
use is_terminal::IsTerminal;
use lazy_static::lazy_static;
use spin_cli::build_info::*;
use spin_cli::commands::{
    build::BuildCommand,
    cloud::{CloudCommand, DeployCommand, LoginCommand},
    doctor::DoctorCommand,
    external::execute_external_subcommand,
    new::{AddCommand, NewCommand},
    plugins::PluginCommands,
    registry::RegistryCommands,
    templates::TemplateCommands,
    up::UpCommand,
    watch::WatchCommand,
};
use spin_redis_engine::RedisTrigger;
use spin_trigger::cli::help::HelpArgsOnlyTrigger;
use spin_trigger::cli::TriggerExecutorCommand;
use spin_trigger_http::HttpTrigger;

#[tokio::main]
async fn main() {
    if let Err(err) = _main().await {
        terminal::error!("{err}");
        print_error_chain(err);
        std::process::exit(1)
    }
}

async fn _main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("watchexec=off".parse()?),
        )
        .with_ansi(std::io::stderr().is_terminal())
        .init();
    SpinApp::parse().run().await
}

fn print_error_chain(err: anyhow::Error) {
    if let Some(cause) = err.source() {
        let is_multiple = cause.source().is_some();
        eprintln!("\nCaused by:");
        for (i, err) in err.chain().skip(1).enumerate() {
            if is_multiple {
                eprintln!("{i:>4}: {}", err)
            } else {
                eprintln!("      {}", err)
            }
        }
    }
}

lazy_static! {
    pub static ref VERSION: String = build_info();
}

/// Helper for passing VERSION to structopt.
fn version() -> &'static str {
    &VERSION
}

/// The Spin CLI
#[derive(Parser)]
#[clap(
    name = "spin",
    version = version()
)]
enum SpinApp {
    #[clap(subcommand, alias = "template")]
    Templates(TemplateCommands),
    New(NewCommand),
    Add(AddCommand),
    Up(UpCommand),
    Cloud(CloudCommand),
    // acts as a cross-level subcommand shortcut -> `spin cloud deploy`
    Deploy(DeployCommand),
    // acts as a cross-level subcommand shortcut -> `spin cloud login`
    Login(LoginCommand),
    #[clap(subcommand, alias = "oci")]
    Registry(RegistryCommands),
    Build(BuildCommand),
    #[clap(subcommand, alias = "plugin")]
    Plugins(PluginCommands),
    #[clap(subcommand, hide = true)]
    Trigger(TriggerCommands),
    #[clap(external_subcommand)]
    External(Vec<String>),
    Watch(WatchCommand),
    Doctor(DoctorCommand),
}

#[derive(Subcommand)]
enum TriggerCommands {
    Http(TriggerExecutorCommand<HttpTrigger>),
    Redis(TriggerExecutorCommand<RedisTrigger>),
    #[clap(name = spin_cli::HELP_ARGS_ONLY_TRIGGER_TYPE, hide = true)]
    HelpArgsOnly(TriggerExecutorCommand<HelpArgsOnlyTrigger>),
}

impl SpinApp {
    /// The main entry point to Spin.
    pub async fn run(self) -> Result<(), Error> {
        match self {
            Self::Templates(cmd) => cmd.run().await,
            Self::Up(cmd) => cmd.run().await,
            Self::New(cmd) => cmd.run().await,
            Self::Add(cmd) => cmd.run().await,
            Self::Cloud(cmd) => cmd.run(SpinApp::command()).await,
            Self::Deploy(cmd) => cmd.run(SpinApp::command()).await,
            Self::Login(cmd) => cmd.run(SpinApp::command()).await,
            Self::Registry(cmd) => cmd.run().await,
            Self::Build(cmd) => cmd.run().await,
            Self::Trigger(TriggerCommands::Http(cmd)) => cmd.run().await,
            Self::Trigger(TriggerCommands::Redis(cmd)) => cmd.run().await,
            Self::Trigger(TriggerCommands::HelpArgsOnly(cmd)) => cmd.run().await,
            Self::Plugins(cmd) => cmd.run().await,
            Self::External(cmd) => execute_external_subcommand(cmd, SpinApp::command()).await,
            Self::Watch(cmd) => cmd.run().await,
            Self::Doctor(cmd) => cmd.run().await,
        }
    }
}

/// Returns build information, similar to: 0.1.0 (2be4034 2022-03-31).
fn build_info() -> String {
    format!("{SPIN_VERSION} ({SPIN_COMMIT_SHA} {SPIN_COMMIT_DATE})")
}
