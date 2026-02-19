use std::{
    env,
    ffi::OsStr,
    fs::{File, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
};

use anyhow::anyhow;
use clap::Parser;
use walkdir::WalkDir;

mod config;

fn find_parent(target: &Path) -> Option<PathBuf> {
    let name = target.file_name()?.to_str()?;
    let dir = target.parent();

    if name == "lib" || name == "main" {
        return None;
    }

    if name != "mod" {
        let dir = dir?;
        for f in ["mod.rs", "lib.rs", "main.rs"] {
            let candidate = dir.join(f);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
        return None;
    }

    let dir = dir?;
    let parent = dir.parent()?;

    for f in ["mod.rs", "lib.rs", "main.rs"] {
        let candidate = parent.join(f);
        if candidate.is_file() {
            return Some(candidate);
        }
    }

    None
}

#[derive(Parser, Debug)]
#[command(name = "cargo-mkrs")]
struct Args {
    /// Module path: foo::bar::baz or foo/bar/baz
    target: String,

    /// Make module public (pub mod)
    #[arg(long)]
    public: bool,
}

fn declare_module(module: &str, parent: &Path, public: bool) -> io::Result<()> {
    let content = std::fs::read_to_string(parent)?;

    let mod_line = format!("mod {module};");
    let public_line = format!("pub mod {module};");

    if content.contains(&mod_line) || content.contains(&public_line) {
        return Ok(());
    }

    let mut f = OpenOptions::new().append(true).open(parent)?;
    writeln!(f, "{}mod {module};", if public { "pub " } else { "" })?;

    Ok(())
}

fn build_target_path(target: &str) -> anyhow::Result<PathBuf> {
    let mut path = env::current_dir()?;
    path.push(target);
    Ok(path)
}

fn extract_module_name(path: &Path) -> anyhow::Result<String> {
    path.file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow!("invalid target name"))
}

fn create_module_file(path: &Path, header: &str) -> anyhow::Result<File> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut f = File::create(path)?;
    write!(f, "{header}")?;
    Ok(f)
}

fn populate_root_module(f: &mut File, parent: &Path, public: bool) -> anyhow::Result<()> {
    for entry in WalkDir::new(parent).into_iter().filter_map(Result::ok) {
        let path = entry.path();

        let is_rs = path.extension() == Some(OsStr::new("rs"));
        let has_mod_rs = path.is_dir() && path.join("mod.rs").is_file();

        if !(is_rs || has_mod_rs) {
            continue;
        }

        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default();

        if matches!(name, "mod" | "lib" | "main") {
            continue;
        }

        writeln!(f, "{}mod {};", if public { "pub " } else { "" }, name)?;
    }

    Ok(())
}

fn run(args: Args) -> anyhow::Result<()> {
    let config = config::get_config();

    let mut path = build_target_path(&args.target)?;
    let target = extract_module_name(&path)?;

    if let Some(parent) = find_parent(&path) {
        declare_module(&target, &parent, args.public)?;
    }

    path.set_extension("rs");

    let mut f = create_module_file(&path, &config.header)?;

    if matches!(target.as_str(), "mod" | "lib" | "main") {
        let parent = path.parent().expect("invalid parent path");
        populate_root_module(&mut f, parent, args.public)?;
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    run(args)
}
