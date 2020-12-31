use async_std::task;
use std::io::Write;
use std::{env, error, fmt, fs, io, path, process};
use structopt::StructOpt;
use surf;

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
        #[structopt(short, long)]
        no_dry_run: bool,
    },
    Which {
        #[structopt(short, long)]
        root: Option<String>,
        version: String,
    },
}

fn command_which(maybe_root: Option<&str>, version: &str) -> Result<(), Box<dyn error::Error>> {
    let versions = get_relative_to_root(maybe_root, "versions")?;
    let entries = easy_read_dir(&versions)?;
    for entry in entries {
        if let Some(path) = entry.file_name() {
            let name = path.to_string_lossy();
            if name.starts_with(version) {
                println!("{}/{}/bin/python3", versions, name);
                return Ok(());
            }
        }
    }
    Err(Box::new(NotFoundError {
        fname: "tarball".into(),
    }))
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

fn find_tarball_or_download(dirname: &str, version: &str) -> Result<String, Box<dyn error::Error>> {
    if let Ok(value) = find_tarball(dirname) {
        return Ok(value);
    }
    fs::create_dir_all(dirname)?;
    let fname = "Python-".to_owned() + version + ".tgz";
    let full_fname = dirname.to_owned() + "/" + &fname;
    let url = "https://www.python.org/ftp/python/".to_owned() + version + "/" + &fname;
    println!("url is {}, full_name is {}", url, full_fname);
    let result = task::block_on(async { download_url_to_file(&url, &full_fname).await });
    match result {
        Err(err) => Err(err),
        Ok(()) => Ok(full_fname),
    }
}

async fn download_url_to_file(url: &str, filename: &str) -> Result<(), surf::Exception> {
    let bytes = surf::get(url).recv_bytes().await?;
    let mut buffer = fs::File::create(filename)?;
    let mut pos = 0;
    while pos < bytes.len() {
        let bytes_written = buffer.write(&bytes[pos..])?;
        pos += bytes_written;
    }
    Ok(())
}

fn find_unpacked(dirname: &str) -> Result<String, Box<dyn error::Error>> {
    let entries = easy_read_dir(dirname)?;
    for entry in entries {
        if let Ok(metadata) = entry.metadata() {
            if metadata.is_dir() {
                let path = entry.file_name().ok_or("cannot get name")?;
                let str_path = path.to_string_lossy().to_string();
                let ret_value = dirname.to_owned() + "/" + &str_path;
                return Ok(ret_value);
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

#[derive(Debug, Clone)]
struct RootNotDirectory {
    name: String,
}

impl fmt::Display for RootNotDirectory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} is not a directory", self.name)
    }
}

impl error::Error for RootNotDirectory {}

fn get_relative_to_root(
    maybe_root: Option<&str>,
    child: &str,
) -> Result<String, Box<dyn error::Error>> {
    let root = match maybe_root {
        Some(pyver_root) => pyver_root.to_owned(),
        None => match env::var("PYVER_ROOT") {
            Ok(directory) => directory,
            Err(_) => env::var("HOME")? + "/.pyver",
        },
    };
    let root_slash = if root.ends_with("/") {
        root
    } else {
        root + "/"
    };
    ensure_directory(&root_slash)?;
    let ret = root_slash + &child;
    ensure_directory(&ret)?;
    Ok(ret)
}

fn ensure_directory(dirname: &str) -> Result<(), Box<dyn error::Error>> {
    match fs::metadata(dirname) {
        Err(error) => {
            if error.kind() == io::ErrorKind::NotFound {
                fs::create_dir(&dirname)?
            } else {
                return Err(Box::new(error));
            }
        }
        Ok(metadata) => {
            if !metadata.is_dir() {
                return Err(Box::new(RootNotDirectory {
                    name: dirname.into(),
                }));
            }
        }
    };
    Ok(())
}

fn find_unpacked_or_unpack(
    maybe_root: Option<&str>,
    version: &str,
) -> Result<String, Box<dyn error::Error>> {
    let child = "sources/".to_owned() + version;
    let relative_child = get_relative_to_root(maybe_root, &child)?;
    if let Ok(unpacked) = find_unpacked(&relative_child) {
        return Ok(unpacked);
    }
    let tarball = find_tarball_or_download(&relative_child, version)?;
    println!("Running tar unpack {}", tarball);
    process::Command::new("tar")
        .args(&["--extract", "--file", &tarball])
        .current_dir(&relative_child)
        .status()?;
    find_unpacked(&relative_child)
}

fn command_build(
    maybe_root: Option<&str>,
    version: &str,
    no_dry_run: bool,
) -> Result<(), Box<dyn error::Error>> {
    let unpacked = find_unpacked_or_unpack(maybe_root, version)?;
    println!("Unpacked {}", unpacked);
    let relative_prefix = "versions/".to_owned() + version;
    let prefix = get_relative_to_root(maybe_root, &relative_prefix)?;
    println!("Prefix {}", prefix);
    if no_dry_run {
        process::Command::new("./configure")
            .args(&["--prefix", &prefix])
            .current_dir(&unpacked)
            .status()?;
        process::Command::new("make")
            .args(&["install"])
            .current_dir(&unpacked)
            .status()?;
    } else {
        println!("Dry run only, not building");
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn error::Error>> {
    let opt = PyVer::from_args();
    match opt {
        PyVer::List { root } => command_list(root.as_deref()),
        PyVer::Cached { root } => command_cached(root.as_deref()),
        PyVer::Which { root, version } => command_which(root.as_deref(), &version),
        PyVer::Build {
            root,
            version,
            no_dry_run,
        } => command_build(root.as_deref(), &version, no_dry_run),
    }?;
    Ok(())
}
