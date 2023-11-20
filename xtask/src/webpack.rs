use std::{path::PathBuf, process::Command};

use clap::Parser;

#[derive(Debug, Parser)]
pub struct Options {
    #[clap(short, long, default_value = "development")]
    pub mode: String,
}

pub fn webpack(_opts: Options) -> Result<(), anyhow::Error> {
    let dir = PathBuf::from("block-io-viz-webapp");

    // Command::new creates a child process which inherits all env variables. This means env
    // vars set by the cargo xtask command are also inherited. RUSTUP_TOOLCHAIN is removed
    // so the rust-toolchain.toml file in the -ebpf folder is honored.

    let status = Command::new("pnpm")
        .args(vec!["exec", "webpack", "--mode", _opts.mode.as_str()])
        .current_dir(dir)
        .env_remove("RUSTUP_TOOLCHAIN")
        .status()
        .expect("failed to build bpf program");
    assert!(status.success());
    Ok(())
}
