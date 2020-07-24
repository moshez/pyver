use std::{env, fs};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "pyver", about = "A Python version manager")]
enum PyVer {
    List {
        #[structopt(short, long)]
        root: Option<String>,
    },
    Cached {
        #[structopt(short, long)]
        root: Option<String>,
    },
    Build {
        #[structopt(short, long)]
        root: Option<String>,
        #[structopt(short, long)]
        version: String,
    },
}

fn command_list(maybe_root: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let versions = get_relative_to_root(maybe_root, "versions".to_string())?;
    print_contents(versions)
}

fn command_cached(maybe_root: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let cached = get_relative_to_root(maybe_root, "sources".to_string())?;
    print_contents(cached)
}

fn print_contents(dirname: String) -> Result<(), Box<dyn std::error::Error>> {
    for entry in fs::read_dir(dirname)? {
        if let Some(path) = entry?.path().file_name() {
            println!("{}", path.to_string_lossy());
        }
    }
    Ok(())
}

fn get_relative_to_root(
    maybe_root: Option<String>,
    child: String,
) -> Result<String, Box<dyn std::error::Error>> {
    let root = match maybe_root {
        Some(pyver_root) => pyver_root,
        None => env::var("PYVER_ROOT")?,
    };
    Ok(root + "/" + &child)
}

fn command_build(
    maybe_root: Option<String>,
    version: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let child = "sources/".to_owned() + &version;
    let relative_child = get_relative_to_root(maybe_root, child)?;

    println!("Pretending to build {}", relative_child);
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = PyVer::from_args();
    match opt {
        PyVer::List { root } => command_list(root),
        PyVer::Cached { root } => command_cached(root),
        PyVer::Build { root, version } => command_build(root, version),
    }?;
    Ok(())
}
