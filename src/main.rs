use std::{
    env,
    ffi::OsStr,
    fs::{self, File, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
};

use anyhow::anyhow;
use clap::Parser;
use walkdir::WalkDir;

mod config;

#[derive(Parser, Debug)]
#[command(version, name = "cargo-mkrs")]
#[command(about = "Cargo subcommand for generating Rust files")]
struct Args {
    target: String,

    #[arg(long)]
    public: bool,
}

/// Walk up the directory tree to find the nearest parent Rust module file
fn find_parent(target: &Path) -> Option<PathBuf> {
    let name = target.file_name()?.to_str()?;
    let dir = target.parent()?;

    if name == "lib" || name == "main" {
        return None;
    }

    if name != "mod" {
        for f in ["mod.rs", "lib.rs", "main.rs"] {
            let candidate = dir.join(f);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
        return None;
    }

    let parent = dir.parent()?;
    for f in ["mod.rs", "lib.rs", "main.rs"] {
        let candidate = parent.join(f);
        if candidate.is_file() {
            return Some(candidate);
        }
    }

    None
}

/// Add a `mod …;` declaration to a parent file if it doesn't already exist
fn declare_module(module: &str, parent: &Path, public: bool) -> io::Result<()> {
    let content = fs::read_to_string(parent)?;

    let mod_line = format!("mod {module};");
    let pub_line = format!("pub mod {module};");

    if content.contains(&mod_line) || content.contains(&pub_line) {
        return Ok(());
    }

    let mut f = OpenOptions::new().append(true).open(parent)?;
    writeln!(f, "{}mod {module};", if public { "pub " } else { "" })?;
    Ok(())
}

/// Build the full path to the target, creating intermediate directories if necessary
fn build_target_path(target: &str) -> anyhow::Result<PathBuf> {
    let mut path = env::current_dir()?;
    path.push(target);

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(path)
}

/// Extract the module name from a path (last component)
fn extract_module_name(path: &Path) -> anyhow::Result<String> {
    path.file_name()
        .or_else(|| path.file_stem())
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow!("invalid target name"))
}

/// Create the new Rust module file with header
fn create_module_file(path: &Path, header: &str) -> anyhow::Result<File> {
    let mut f = File::create(path)?;
    write!(f, "{header}")?;
    Ok(f)
}

/// Populate a root module (mod.rs, lib.rs, main.rs) with submodules
fn populate_root_module(f: &mut File, parent: &Path, public: bool) -> anyhow::Result<()> {
    for entry in WalkDir::new(parent)
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .filter_map(Result::ok)
    {
        let path = entry.path();

        let is_rs_file = path.extension() == Some(OsStr::new("rs"));
        let has_mod_rs = path.is_dir() && path.join("mod.rs").is_file();

        if !(is_rs_file || has_mod_rs) {
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
