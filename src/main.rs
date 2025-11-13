use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::env;

const ILI_PATH : &str = "C:\\ProgramData\\ILI";

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_help();
        return;
    }

    let command = args[1].as_str();
    let libs_dir = libs_dir();

    match command {
        "install" => {
            if let Some(name) = args.get(2) {
                install(name, &libs_dir);
            } else {
                eprintln!("Usage: ili install <name>");
            }
        }
        "update" => {
            if let Some(name) = args.get(2) {
                update(name, &libs_dir);
            } else {
                eprintln!("Usage: ili update <name>");
            }
        }
        "remove" => {
            if let Some(name) = args.get(2) {
                remove(name, &libs_dir);
            } else {
                eprintln!("Usage: ili remove <name>");
            }
        }
        "where" => {
            if let Some(name) = args.get(2) {
                show_path(name, &libs_dir);
            } else {
                eprintln!("Usage: ili where <name>");
            }
        }
        "sync" => {
            sync_registry();
        }
        _ => print_help(),
    }
}

fn print_help() {
    println!(
        "Usage: ili <command> [args]
Commands:
  install <name>   Install a library from the registry
  update <name>    Update an installed library
  remove <name>    Remove a library
  where <name>     Show installation path
  sync             Update local copy of registry
"
    );
}

fn libs_dir() -> PathBuf {
    let path = PathBuf::from(ILI_PATH).join("libs");
    println!("Using libs directory: {}", path.display());
    return path;
}

fn install(name: &str, libs_dir: &Path) {
    let registry = ensure_registry();
    let content = fs::read_to_string(&registry).unwrap_or_default();

    let repo = find_repo(&content, name);
    if repo.is_empty() {
        eprintln!("No entry found for '{}'", name);
        return;
    }

    let dest = libs_dir.join(name);
    if dest.exists() {
        println!("'{}' already installed at {}", name, dest.display());
        return;
    }

    fs::create_dir_all(&libs_dir).unwrap();
    println!("Cloning {} -> {}", repo, dest.display());

    let status = Command::new("git")
        .args(["clone", &repo, dest.to_str().unwrap()])
        .status()
        .expect("Failed to run git clone");

    if status.success() {
        println!("Installed '{}'", name);
    } else {
        eprintln!("Git clone failed for '{}'", name);
    }
}

fn update(name: &str, libs_dir: &Path) {
    let path = libs_dir.join(name);
    if !path.exists() {
        eprintln!("'{}' is not installed", name);
        return;
    }

    println!("Updating '{}'...", name);
    let status = Command::new("git")
        .args(["-C", path.to_str().unwrap(), "pull"])
        .status()
        .expect("Failed to run git pull");

    if status.success() {
        println!("Updated '{}'", name);
    } else {
        eprintln!("Update failed for '{}'", name);
    }
}

fn remove(name: &str, libs_dir: &Path) {
    let path = libs_dir.join(name);
    if !path.exists() {
        eprintln!("'{}' not installed", name);
        return;
    }
    fs::remove_dir_all(&path).unwrap();
    println!("Removed '{}'", name);
}

fn show_path(name: &str, libs_dir: &Path) {
    let path = libs_dir.join(name);
    if path.exists() {
        println!("'{}' installed at {}", name, path.display());
    } else {
        println!("'{}' not installed", name);
    }
}

fn ensure_registry() -> PathBuf {
    let local = PathBuf::from(ILI_PATH);
    let registry_file = local.join("registry.txt");

    if !local.exists() {
        println!("Cloning registry...");
        clone_registry(&local);
    } else {
        // If registry exists, make sure it's fresh
        println!("Updating local registry...");
        let _ = Command::new("git")
            .args(["-C", local.to_str().unwrap(), "pull"])
            .status();
    }

    registry_file
}

fn clone_registry(path: &Path) {
    let registry_repo = "https://github.com/I-had-a-bad-idea/ILI.git";

    let status = Command::new("git")
        .args(["clone", registry_repo, path.to_str().unwrap()])
        .status()
        .expect("Failed to clone registry repo");

    if !status.success() {
        eprintln!("Failed to clone registry repository");
    }
}

fn sync_registry() {
    let path = PathBuf::from(ILI_PATH);
    if path.exists() {
        println!("Pulling latest registry...");
        let _ = Command::new("git")
            .args(["-C", path.to_str().unwrap(), "pull"])
            .status();
    } else {
        println!("No local registry found, cloning fresh...");
        clone_registry(&path);
    }
}

fn find_repo(content: &str, name: &str) -> String {
    for line in content.lines() {
        if let Some((n, url)) = line.split_once('=') {
            if n.trim() == name {
                return url.trim().to_string();
            }
        }
    }
    String::new()
}
