use std::{
    env,
    path::{Path, PathBuf},
};

use clap::{Parser, ValueEnum};
use xshell::{cmd, Shell};

#[derive(Parser)]
#[command(author, about, long_about = None)]
struct Args {
    #[arg(long, help = "Perform a clean installation of npm packages")]
    ci: bool,
    #[arg(long, help = "Install the generated .vsix")]
    install: bool,
    #[arg(long, help = "which platform to compile against")]
    platform: Platform,
}

#[derive(ValueEnum, Clone)]
enum Platform {
    All,
    Windows,
    Unix,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let sh = &Shell::new()?;
    let dir = project_root();
    sh.change_dir(dir);

    let targets = get_targets(&args);
    sh.create_dir("./lsp-extension/server")?;
    for target in targets {
        cmd!(sh, "cargo build -p lsp-rs --release --target {target}").run()?;
        if target.contains("windows") {
            sh.copy_file(
                format!("./target/{target}/release/lsp-rs.exe"),
                "./lsp-extension/server/ksp-cfg-lsp.exe",
            )?;
        } else {
            sh.copy_file(
                format!("./target/{target}/release/lsp-rs"),
                "./lsp-extension/server/ksp-cfg-lsp",
            )?;
        }
    }
    sh.change_dir("./lsp-extension");
    if cfg!(target_family = "unix") {
        if args.ci {
            cmd!(sh, "npm ci").run()?;
        }
        cmd!(sh, "npm run package").run()?;
        if args.install {
            cmd!(sh, "code --install-extension ksp-cfg-lsp.vsix").run()?;
        }
    } else {
        if args.ci {
            cmd!(sh, "cmd.exe /c npm ci").run()?;
        }
        cmd!(sh, "cmd.exe /c npm run package").run()?;
        if args.install {
            cmd!(sh, "cmd.exe /c code --install-extension ksp-cfg-lsp.vsix").run()?;
        }
    }
    Ok(())
}

fn get_targets(args: &Args) -> Vec<&str> {
    let mut targets = vec![];
    if matches!(args.platform, Platform::Unix | Platform::All) {
        targets.push("x86_64-unknown-linux-gnu");
    }
    if matches!(args.platform, Platform::Windows | Platform::All) {
        if cfg!(target_family = "unix") {
            targets.push("x86_64-pc-windows-gnu");
        } else {
            targets.push("x86_64-pc-windows-msvc");
        }
    }
    targets
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
