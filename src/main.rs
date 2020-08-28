use std::{env, error, fmt, fs, path, process};
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

fn command_list(maybe_root: Option<&str>) -> Result<(), Box<dyn error::Error>> {
    let versions = get_relative_to_root(maybe_root, "versions")?;
    print_contents(&versions)
}

fn command_cached(maybe_root: Option<&str>) -> Result<(), Box<dyn error::Error>> {
    let cached = get_relative_to_root(maybe_root, "sources")?;
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

fn find_unpacked(dirname: &str) -> Result<String, Box<dyn error::Error>> {
    let entries = easy_read_dir(dirname)?;
    for entry in entries {
        if let Ok(metadata) = entry.metadata() {
            if metadata.is_dir() {
                let path = entry.file_name().ok_or("cannot get name")?;
		return Ok(path.to_string_lossy().to_string());
            }
        }
    }
    Err(Box::new(NotFoundError {
        fname: "unpacked".into(),
    }))
}

fn easy_read_dir(
    dirname: &str,
) -> Result<impl Iterator<Item = path::PathBuf>, Box<dyn error::Error>> {
    let entries = fs::read_dir(dirname)?
        .filter_map(|res| res.ok())
        .map(|res| res.path());
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
    maybe_root: Option<&str>,
    child: &str,
) -> Result<String, Box<dyn error::Error>> {
    let root = match maybe_root {
        Some(pyver_root) => pyver_root.to_owned(),
        None => env::var("PYVER_ROOT")?,
    };
    let root_slash = if root.ends_with("/") {
        root
    } else {
        root + "/"
    };
    Ok(root_slash + &child)
}

fn command_build(maybe_root: Option<&str>, version: &str) -> Result<(), Box<dyn error::Error>> {
    let child = "sources/".to_owned() + version;
    let relative_child = get_relative_to_root(maybe_root, &child)?;
    let tarball = find_tarball(&relative_child)?;

    println!("Running tar unpack {}", tarball);

    process::Command::new("tar")
        .args(&["--extract", "--file", &tarball])
        .current_dir(&relative_child)
        .status()?;

    let unpacked = relative_child.to_owned() + "/" + &find_unpacked(&relative_child)?;
    println!("Unpacked {}", unpacked);
    let prefix = get_relative_to_root(maybe_root, &("versions/".to_owned() + version))?;
    println!("Prefix {}", prefix);
    process::Command::new("./configure")
        .args(&["--prefix", &prefix])
        .current_dir(&unpacked)
        .status()?;
    process::Command::new("make")
        .args(&["install"])
        .current_dir(&unpacked)
        .status()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn error::Error>> {
    let opt = PyVer::from_args();
    match opt {
        PyVer::List { root } => command_list(root.as_deref()),
        PyVer::Cached { root } => command_cached(root.as_deref()),
        PyVer::Build { root, version } => command_build(root.as_deref(), &version),
    }?;
    Ok(())
}
