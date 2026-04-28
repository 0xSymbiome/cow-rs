use clap::{Parser, Subcommand};
use policy_maintainer::{check_stub, diagnostics::OutputMode};

#[derive(Debug, Parser)]
#[command(
    name = "policy-maintainer",
    version,
    about = "Run cow-rs policy maintenance checks."
)]
struct Cli {
    /// Emit diagnostics as newline-delimited JSON.
    #[arg(long, global = true)]
    json: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Run the policy-maintainer skeleton smoke check.
    #[command(name = "check-stub")]
    CheckStub(check_stub::Args),
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let output_mode = OutputMode::from_json(cli.json);

    match cli.command {
        Command::CheckStub(args) => check_stub::run(args, output_mode),
    }
}
