use std::io::{self, Write};

use anyhow::Context;

use crate::diagnostics::{Diagnostic, OutputMode};

pub const STUB_DIAGNOSTIC_CODE: &str = "PM0000";

#[derive(Debug, clap::Args)]
pub struct Args {}

pub fn run(args: Args, output_mode: OutputMode) -> anyhow::Result<()> {
    let mut stdout = io::stdout().lock();
    run_with_writer(args, output_mode, &mut stdout)
}

pub fn run_with_writer(
    _args: Args,
    output_mode: OutputMode,
    writer: &mut impl Write,
) -> anyhow::Result<()> {
    diagnostic()
        .emit(output_mode, writer)
        .context("failed to emit check-stub diagnostic")
}

pub fn diagnostic() -> Diagnostic {
    Diagnostic::info(
        STUB_DIAGNOSTIC_CODE,
        "policy-maintainer skeleton operational",
    )
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::{Args, STUB_DIAGNOSTIC_CODE, run_with_writer};
    use crate::diagnostics::OutputMode;

    #[test]
    fn check_stub_emits_text_diagnostic() {
        let mut output = Vec::new();

        run_with_writer(Args {}, OutputMode::Text, &mut output).unwrap();

        assert_eq!(
            String::from_utf8(output).unwrap(),
            format!("info {STUB_DIAGNOSTIC_CODE}: policy-maintainer skeleton operational\n")
        );
    }

    #[test]
    fn check_stub_emits_json_diagnostic() {
        let mut output = Vec::new();

        run_with_writer(Args {}, OutputMode::Json, &mut output).unwrap();

        let value: Value = serde_json::from_slice(&output).unwrap();
        assert_eq!(value["level"], "info");
        assert_eq!(value["code"], STUB_DIAGNOSTIC_CODE);
        assert_eq!(value["message"], "policy-maintainer skeleton operational");
    }
}
