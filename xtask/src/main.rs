use std::{
    env,
    path::{Path, PathBuf},
};

use clap::Parser;
use xshell::{cmd, Shell};

#[derive(Parser)]
#[command(author, about, long_about = None)]
struct Args {
    #[arg(long, help = "Perform a clean installation of npm packages")]
    ci: bool,
    #[arg(long, help = "Install the generated .vsix")]
    install: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let sh = &Shell::new()?;
    let dir = project_root();
    sh.change_dir(dir);
    cmd!(sh, "cargo build -p lsp-rs --release").run()?;
    sh.create_dir("./lsp-extension/server")?;
    sh.copy_file(
        "./target/release/lsp-rs.exe",
        "./lsp-extension/server/lsp-rs.exe",
    )?;
    sh.change_dir("./lsp-extension");
    if args.ci {
        cmd!(sh, "cmd.exe /c npm ci").run()?;
    }
    cmd!(sh, "cmd.exe /c npm run package").run()?;
    if args.install {
        cmd!(sh, "cmd.exe /c code --install-extension ksp-cfg-lsp.vsix").run()?;
    }
    Ok(())
}

fn project_root() -> PathBuf {
    Path::new(
        &env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| env!("CARGO_MANIFEST_DIR").to_owned()),
    )
    .ancestors()
    .nth(1)
    .unwrap()
    .to_path_buf()
}
