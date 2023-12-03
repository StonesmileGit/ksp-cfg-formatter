use std::{
    env,
    error::Error,
    fmt::Display,
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
    #[arg(long, help = "SemVer version", value_parser = Version::parse)]
    version: Version,
}

#[derive(ValueEnum, Clone)]
enum Platform {
    All,
    Windows,
    Unix,
}

#[derive(Clone)]
struct Version {
    major: usize,
    minor: usize,
    patch: usize,
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[derive(Debug)]
enum ParseError {
    Major,
    Minor,
    Patch,
    TooMany,
}

impl Error for ParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }

    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Failed to parse version at {}",
            match self {
                ParseError::Major => "MAJOR",
                ParseError::Minor => "MINOR",
                ParseError::Patch => "PATCH",
                ParseError::TooMany => "TOO MANY ARGS",
            }
        )
    }
}

impl Version {
    fn parse(s: &str) -> Result<Self, Box<dyn Error + Send + Sync + 'static>> {
        let mut splits = s.split('.');
        match splits.clone().count() {
            0 => return Err(Box::new(ParseError::Major)),
            1 => return Err(Box::new(ParseError::Minor)),
            2 => return Err(Box::new(ParseError::Patch)),
            3 => (),
            _ => return Err(Box::new(ParseError::TooMany)),
        }
        Ok(Version {
            major: splits.next().unwrap().parse()?,
            minor: splits.next().unwrap().parse()?,
            patch: splits.next().unwrap().parse()?,
        })
    }
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let sh = &Shell::new()?;
    let dir = project_root();
    sh.change_dir(dir);

    let targets = get_targets(&args);
    sh.change_dir("./lsp-extension");
    let ver = args.version.to_string();
    if cfg!(target_family = "unix") {
        cmd!(sh, "npm version {ver} --allow-same-version").run()?;
    } else {
        cmd!(sh, "cmd.exe /c npm version {ver} --allow-same-version").run()?;
    }
    if cfg!(target_family = "unix") {
        cmd!(sh, "cargo set-version -p lsp-rs {ver}").run()?;
    } else {
        cmd!(sh, "cmd.exe /c cargo set-version -p lsp-rs {ver}").run()?;
    }
    sh.create_dir("./server")?;
    for target in targets {
        cmd!(sh, "cargo build -p lsp-rs --release --target {target}").run()?;
        if target.contains("windows") {
            sh.copy_file(
                format!("../target/{target}/release/lsp-rs.exe"),
                "./server/ksp-cfg-lsp.exe",
            )?;
        } else {
            sh.copy_file(
                format!("../target/{target}/release/lsp-rs"),
                "./server/ksp-cfg-lsp",
            )?;
        }
    }
    if args.ci {
        if cfg!(target_family = "unix") {
            cmd!(sh, "npm ci").run()?;
        } else {
            cmd!(sh, "cmd.exe /c npm ci").run()?;
        }
    }
    if cfg!(target_family = "unix") {
        cmd!(sh, "npm run package").run()?;
    } else {
        cmd!(sh, "cmd.exe /c npm run package").run()?;
    }
    if args.install {
        if cfg!(target_family = "unix") {
            cmd!(sh, "code --install-extension ksp-cfg-lsp-{ver}.vsix").run()?;
        } else {
            cmd!(
                sh,
                "cmd.exe /c code --install-extension ksp-cfg-lsp-{ver}.vsix"
            )
            .run()?;
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
