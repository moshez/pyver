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

fn command_list(root: Option<String>) {
    let pyver_root = root.ok_or(env::var("PYVER_ROOT")).unwrap();
    for entry in fs::read_dir(pyver_root).unwrap() {
        let path = entry.unwrap().path();
        println!("{}", path.file_name().unwrap().to_str().unwrap());
    }
}

fn main() {
    let opt = PyVer::from_args();
    match opt {
        PyVer::List { root } => command_list(root),
    };
}
