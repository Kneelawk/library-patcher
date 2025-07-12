use anyhow::Context;
use clap::Parser;
use glob::Pattern;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

#[derive(Parser)]
struct Args {
    /// The path that library files are copied to
    #[arg(short, long)]
    output: PathBuf,
    /// The library files to copy (globs accepted)
    #[arg(short, long)]
    target: Vec<String>,
    /// The library files to exclude (globs accepted)
    #[arg(short, long)]
    exclude: Vec<String>,
    /// Only print names of files to be copied without actually copying them
    #[arg(short, long)]
    dry_run: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let mut to_copy = BTreeSet::new();
    let root = PathBuf::from("/");

    let excludes = args
        .exclude
        .iter()
        .map(|s| Pattern::new(s))
        .collect::<Result<Vec<_>, _>>()
        .context("Parsing exclude globs")?;

    for target in args.target.iter() {
        for glob_result in
            glob::glob(target).with_context(|| format!("Parsing target glob: {}", target))?
        {
            if let Ok(file) = glob_result {
                let result = lddtree::DependencyAnalyzer::new(root.clone())
                    .analyze(&file)
                    .with_context(|| format!("Analying: {:?}", &file))?;
                for (_name, library) in result.libraries {
                    if check(&excludes, &library.path) {
                        to_copy.insert(library.path);
                    }
                }
            }
        }
    }

    println!("Copying files:");
    for file in to_copy {
        if let Some(file_name) = file.file_name() {
            let output = args.output.join(file_name);
            println!("  {:?} to {:?}", &file, &output);
            if !args.dry_run {
                std::fs::copy(&file, &output)
                    .with_context(|| format!("Copying {:?} to {:?}", &file, &output))?;
            }
        }
    }

    Ok(())
}

fn check(excludes: &[Pattern], path: impl AsRef<Path>) -> bool {
    for exclude in excludes {
        if exclude.matches_path(path.as_ref()) {
            return false;
        }
    }
    true
}
