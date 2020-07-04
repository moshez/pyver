use std::{env, fs};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "pyver", about = "A Python version manager")]
enum PyVer {
    List {
        #[structopt(short, long)]
        root: Option<String>,
    },
}

fn command_list(maybe_root: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let pyver_root = match maybe_root {
        Some(pyver_root) => pyver_root,
        None => env::var("PYVER_ROOT")?,
    };
    for entry in fs::read_dir(pyver_root)? {
        if let Some(path) = entry?.path().file_name() {
            println!("{}", path.to_string_lossy());
        }
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = PyVer::from_args();
    match opt {
        PyVer::List { root } => command_list(root)?,
    };
    Ok(())
}
