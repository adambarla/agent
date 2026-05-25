use std::process::ExitCode;

fn main() -> ExitCode {
    match agent::harness::run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            log::error!("{error:?}");
            eprintln!("The chat request failed. Details were written to the log file.");
            ExitCode::FAILURE
        }
    }
}
