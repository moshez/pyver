use std::{env, error, fmt, fs, process, path};
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

fn command_list(maybe_root: Option<String>) -> Result<(), Box<dyn error::Error>> {
    let versions = get_relative_to_root(maybe_root, "versions".to_string())?;
    print_contents(&versions)
}

fn command_cached(maybe_root: Option<String>) -> Result<(), Box<dyn error::Error>> {
    let cached = get_relative_to_root(maybe_root, "sources".to_string())?;
    print_contents(&cached)
}

#[derive(Debug, Clone)]
struct NotFoundError {
    fname: String,
}

impl fmt::Display for NotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Could not find {}", self.fname)
    }
}

impl error::Error for NotFoundError {}

fn find_tarball(dirname: &str) -> Result<String, Box<dyn error::Error>> {
    let entries = easy_read_dir(dirname)?;
    for entry in entries {
        if let Some(path) = entry.file_name() {
            let name = path.to_string_lossy();
            if name.ends_with(".tgz") || name.ends_with(".xz") {
                let ret_value = dirname.to_owned() + "/" + &name;
                return Ok(ret_value);
            }
        }
    }
    Err(Box::new(NotFoundError {
        fname: "tarball".into(),
    }))
}


fn easy_read_dir(dirname: &str) -> Result<impl Iterator<Item=path::PathBuf>, Box<dyn error::Error>> {
    let entries = fs::read_dir(dirname)?
                    .filter_map(|res| res.ok()).map(|res| res.path());
    Ok(entries)
}

fn print_contents(dirname: &str) -> Result<(), Box<dyn error::Error>> {
    let entries = easy_read_dir(dirname)?;
    for entry in entries {
        if let Some(path) = entry.file_name() {
            println!("{}", path.to_string_lossy());
        }
    }
    Ok(())
}

fn get_relative_to_root(
    maybe_root: Option<String>,
    child: String,
) -> Result<String, Box<dyn error::Error>> {
    let root = match maybe_root {
        Some(pyver_root) => pyver_root,
        None => env::var("PYVER_ROOT")?,
    };
    let root_slash = if root.ends_with("/") {
        root
    } else {
        root + "/"
    };
    Ok(root_slash + &child)
}

fn command_build(maybe_root: Option<String>, version: String) -> Result<(), Box<dyn error::Error>> {
    let child = "sources/".to_owned() + &version;
    let relative_child = get_relative_to_root(maybe_root, child)?;
    let tarball = find_tarball(&relative_child)?;

    println!("Running tar unpack {}", tarball);

    process::Command::new("tar")
        .args(&["--extract", "--file", &tarball])
        .current_dir(&relative_child)
        .status()?;

    println!("Pretending to build in {}", relative_child);
    Ok(())
}

fn main() -> Result<(), Box<dyn error::Error>> {
    let opt = PyVer::from_args();
    match opt {
        PyVer::List { root } => command_list(root),
        PyVer::Cached { root } => command_cached(root),
        PyVer::Build { root, version } => command_build(root, version),
    }?;
    Ok(())
}
