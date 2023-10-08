use clap::Parser;
use duct::cmd;
use std::error::Error;

#[derive(Parser, Clone, Debug, PartialEq, Eq)]
#[command()]
pub enum Args {
    /// Do the full CI test run
    Ci,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    match args {
        Args::Ci => {
            cmd!("cargo", "test").run()?;
            cmd!("cargo", "test", "--manifest-path", "example/Cargo.toml").run()?;
            cmd!("cargo", "test", "--manifest-path", "tests/bevy/Cargo.toml").run()?;
        }
    }
    Ok(())
}
