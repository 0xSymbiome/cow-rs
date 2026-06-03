use clap::{Parser, Subcommand, ValueEnum};
use serde_json::json;
use validation_smoke::registry_confirm;

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum OutputFormat {
    Text,
    Json,
}

#[derive(Debug, Parser)]
#[command(
    about = "Confirm CoW Protocol deployment provenance against live chain bytecode."
)]
struct Cli {
    #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
    format: OutputFormat,

    #[command(subcommand)]
    command: SmokeCommand,
}

#[derive(Debug, Subcommand)]
enum SmokeCommand {
    #[command(about = "Confirm deployment provenance against live chain bytecode")]
    RegistryConfirm(registry_confirm::RegistryConfirmArgs),
}

fn emit_command_error(format: OutputFormat, code: &str, message: &str) {
    match format {
        OutputFormat::Text => eprintln!("error {code}: {message}"),
        OutputFormat::Json => eprintln!(
            "{}",
            serde_json::to_string(&json!({
                "level": "error",
                "code": code,
                "message": message,
            }))
            .expect("error diagnostic should serialize")
        ),
    }
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        SmokeCommand::RegistryConfirm(args) => match registry_confirm::run(args) {
            Ok(report) => {
                match cli.format {
                    OutputFormat::Text => println!("{}", report.render_text()),
                    OutputFormat::Json => println!(
                        "{}",
                        serde_json::to_string_pretty(&report)
                            .expect("registry-confirm report should serialize")
                    ),
                }
                std::process::exit(report.exit_code());
            }
            Err(error) => {
                emit_command_error(cli.format, "VS10001", &error.to_string());
                std::process::exit(1);
            }
        },
    }
}
