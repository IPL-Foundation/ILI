use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::env;

const ILI_PATH : &str = "C:\\ProgramData\\ILI"; // Change as needed

#[derive(Debug)]
struct Library {
    name: String,
    version: String,
    entry: String,
    dependencies: Vec<String>,
}

fn load_library_json(path: &Path) -> Option<Library> {
    let file = path.join("Library.json");
    let raw = fs::read_to_string(&file).ok()?;

    let mut name = String::new();
    let mut version = String::new();
    let mut entry = String::new();
    let mut dependencies = Vec::new();

    for line in raw.lines() {
        let l = line.trim();

        if l.starts_with("\"name\"") {
            name = extract_string(l)?;
        } else if l.starts_with("\"version\"") {
            version = extract_string(l)?;
        } else if l.starts_with("\"entry\"") {
            entry = extract_string(l)?;
        } else if l.starts_with("\"dependencies\"") {
            dependencies = extract_array(l, &raw)?;
        }
    }

    Some(Library { name, version, entry, dependencies })
}

fn extract_string(line: &str) -> Option<String> {
    let start = line.find('"')?; // Where it starts
    let rest = &line[start+1..];
    let mid = rest.find('"')?; // End of key
    let rest = &rest[mid+1..];
    let value_start = rest.find('"')?; // Start of value
    let rest = &rest[value_start+1..];
    let value_end = rest.find('"')?; // End of value
    Some(rest[..value_end].to_string()) // Extracted value
}

fn extract_array(_line: &str, full: &str) -> Option<Vec<String>> {
    let start = full.find('[')?; // Start of array
    let end = full.find(']')?; // End of array
    let inside = &full[start+1..end]; // Inside the brackets
    let mut out = Vec::new();
    for part in inside.split(',') { // Split by commas
        let t = part.trim();
        if t.starts_with('"') && t.ends_with('"') { // Is a string
            out.push(t[1..t.len()-1].to_string()); // Remove quotes
        }
    }
    Some(out)
}

fn main() {
    let args: Vec<String> = env::args().collect(); // Get command-line arguments
    if args.len() < 2 {
        print_help();
        return;
    }

    let command = args[1].as_str(); // First argument is command
    let libs_dir = libs_dir(); // Get libs directory

    // Match command
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
            ensure_registry();
        }
        _ => print_help(),
    }
}

// Print help message
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
// Get the libs directory path
fn libs_dir() -> PathBuf {
    let path = PathBuf::from(ILI_PATH).join("libs");
    println!("Using libs directory: {}", path.display());
    return path;
}
// Install a library by name
fn install(name: &str, libs_dir: &Path) {
    let registry = ensure_registry();
    let content = fs::read_to_string(&registry).unwrap_or_default(); // Read registry

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

    fs::create_dir_all(&libs_dir).unwrap(); // Ensure libs directory exists
    println!("Cloning {} -> {}", repo, dest.display());

    // Clone the repository
    let status = Command::new("git")
        .args(["clone", &repo, dest.to_str().unwrap()])
        .status()
        .expect("Failed to run git clone");

    if !status.success() {
        eprintln!("Git clone failed for '{}'", name);
        return;
    }

    // Validate Library.json
    match load_library_json(&dest) {
        Some(lib) => {
            println!("Installed '{}'", lib.name);
            println!("Version: {}", lib.version);
            println!("Entry point: {}", lib.entry);

            if !lib.dependencies.is_empty() {
                println!("Dependencies: {:?}", lib.dependencies);

                for dep in &lib.dependencies { // Install dependencies
                    install(dep, libs_dir);
                }
            }
        }
        None => {
            eprintln!("Invalid library: missing or malformed Library.json");
            fs::remove_dir_all(&dest).unwrap();
            return;
        }
    }
}

// Update an installed library
fn update(name: &str, libs_dir: &Path) {
    let path = libs_dir.join(name);
    if !path.exists() {
        eprintln!("'{}' is not installed", name);
        return;
    }

    println!("Updating '{}'...", name);
    // Pull latest changes
    let status = Command::new("git")
        .args(["-C", path.to_str().unwrap(), "pull"])
        .status()
        .expect("Failed to run git pull");

    if !status.success() {
        eprintln!("Update failed for '{}'", name);
        return;
    }

    // Reload Library.json
    match load_library_json(&path) {
        Some(lib) => {
            println!("Updated '{}'", lib.name);
            println!("Version: {}", lib.version);

            if !lib.dependencies.is_empty() {
                println!("Dependencies: {:?}", lib.dependencies);
                for dep in &lib.dependencies { // Update dependencies
                    update(dep, libs_dir);
                }
            }
        }
        None => {
            eprintln!("Warning: '{}' updated but Library.json is invalid!", name);
        }
    }
}
// Remove an installed library
fn remove(name: &str, libs_dir: &Path) {
    let path = libs_dir.join(name);
    if !path.exists() {
        eprintln!("'{}' not installed", name);
        return;
    }
    fs::remove_dir_all(&path).unwrap(); // Remove the directory
    println!("Removed '{}'", name);
}
// Show installation path of a library
fn show_path(name: &str, libs_dir: &Path) {
    let path = libs_dir.join(name);
    if path.exists() {
        println!("'{}' installed at {}", name, path.display());
    } else {
        println!("'{}' not installed", name);
    }
}
// Ensure the registry is present and up-to-date
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
// Clone the registry repository
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

// Find repository URL for a given library name
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
