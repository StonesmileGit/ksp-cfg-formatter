use std::{
    env,
    path::{Path, PathBuf},
};

use xshell::{cmd, Shell};

fn main() -> anyhow::Result<()> {
    let sh = &Shell::new()?;
    let dir = project_root();
    sh.change_dir(dir);
    cmd!(sh, "cargo build -p lsp-rs --release").run()?;
    sh.create_dir("./client/server")?;
    sh.copy_file(
        "./target/release/lsp-rs.exe",
        "./lsp-extension/server/lsp-rs.exe",
    )?;
    sh.change_dir("./lsp-extension");
    cmd!(sh, "cmd.exe /c npm run package").run()?;
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
