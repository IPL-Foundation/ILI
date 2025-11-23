# ILI (IPL Library Installer)

ILI is the package manager for [IPL](https://github.com/I-had-a-bad-idea/IPL). It installs, updates and removes IPL libraries.

## Installation

```bash
git clone https://github.com/I-had-a-bad-idea/ILI.git C:\ProgramData\ILI
cargo install --path C:\ProgramData\ILI
```

After installation, the `ili` command becomes available system-wide.

## Commands

Call commands via `ili command args`

### `install <name>`

Install a library from the registry.
Downloads the library, extracts it into the IPL library directory, and install dependencies.

### `update (<name>)`

Update an installed library and its dependencies, if name is given. Else updates all libraries.

### `remove <name>`

Remove a previously installed library, including its metadata and local files.

### `where <name>`

Print the installation path of a library.

### `sync`

Pull the latest registry index so `ili` knows about new/updated libraries.

## Registry Structure

ILI expects the registry to contain entries describing libraries (name, version, source URL, metadata). Libraries are fetched from the URLs listed in the registry.

## Contributing a Library

1. **Use the example library as a template**
   [https://github.com/I-had-a-bad-idea/Example-IPL-Library](https://github.com/I-had-a-bad-idea/Example-IPL-Library)

2. **Create a valid `Library.json`**
   It must include:

   * `name`
   * `version`
   * `entry` (the main IPL file)
   * `dependencies` (list of all dependencies to install)

3. **Add it to the registry**
   Submit a pull request adding a new entry for your library in the registry file.
   Follow the example already in the registry file (you need a name, and the url to your git repo).

> Note: Currently classes inside libraries are not supported by IPL

