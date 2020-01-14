[![Actions Status](https://github.com/stevepentland/spinup/workflows/Rust/badge.svg)](https://github.com/stevepentland/spinup/actions) [![Dependabot Status](https://api.dependabot.com/badges/status?host=github&repo=stevepentland/spinup&identifier=220822410)](https://dependabot.com) [![codecov](https://codecov.io/gh/stevepentland/spinup/branch/master/graph/badge.svg)](https://codecov.io/gh/stevepentland/spinup)

# Spinup

## Special Note

Spinup is currently beta software

## Background

Spinup is a program designed to make setting up newly installed Linux machines easy and repeatable. There is no complicated software to install, just a single application and a configuration file that can be in json, toml, or yaml format. It comes from my regular distro
hopping and always wanting to get back to a working system. I had originally been using a shell script in a gist, but wanted something a bit more robust.

It will perform system upgrades, install packages via your package manager, download files from the web, run custom commands, and install snap packages. All actions are defined in a configuration file and passed to the application at runtime.

## Getting Started

Currently, an initial build is provided, but it can also be quickly built by cloning this repo and running `cargo build --release` .

Spinup will build on current stable rust, with a version of 1.39 or greater.

Once you have the `spinup` binary, you can put it onto a newly installed machine and run it. In the future, builds will be populated in the [Releases](https://github.com/stevepentland/spinup/releases) page.

## Running Spinup

When you have a spinup binary, you simply have to run it while passing in a path to a configuration file:

``` 
./spinup my-config.yml
```

To see all available options, run spinup with the `-h` or `--help` argument.

## Configuration Files

Spinup plays a set of instructions that are provided via a configuration file. Examples of configuration files can be found in the [examples](https://github.com/stevepentland/spinup/tree/master/examples) directory in the project.

### Main Configuration Items

The following items can be defined at the top level of the configuration file:

* [package_list](#specifying-packages) 
  + This is the main set of packages you want to install from your distro's package manager
* [file_downloads](#downloading-files) 
  + These are files that you want to have downloaded to your system. These could be config files, fonts, etc
* [snaps](#installing-snap-packages) 
  + Snap packages to install, currently requires that you install `snapd` in the package list or are on a system with it already present 
* `update_system` 
  + Whether to run your system's update/upgrade commands before starting the install process
  + **Note:** Implementation of this feature is currently in progress

All items are optional, and if not defined, will just be skipped.

### Specifying Packages

Packages are defined within the `package_list` configuration element. There are two different fields for the packages section:

* `base_packages` 
  + This is just an optional list of strings for package names to install
  + These names will be used as-is, so they should be names that are universal across distros
* [distro_packages](#distro-packages)
  + Packages that have names specific for a given distro
  + This is a list of objects detailing the names of the packages and their platform

All packages listed will be passed to the default package manager for the platform. Auto-confirmation will be specified, meaning no confirmation will be requested at run time. If the install process requires root (which it generally will), the user will be prompted to authorize a `sudo` session.

#### Distro Packages

To accommodate packages which have different names based on the target distro, the `distro_package` objects allow you to specify those packages. Each object in the collection has two fields:

* `target_os` 
  + This is the name of the distro, as defined in the `/etc/os-release` file `ID` or `ID_LIKE` field.
* `packages` 
  + A list of the packages to install

### Downloading Files

The `file_downloads` collection will allow you to obtain arbitrary files and run a custom command after the downloads complete. Each object in the collection has the following fields:

* `base_dir` 
  + This is the directory to download this set of files _into_
* `after_complete` 
  + A [Custom Command](#custom-commands) to run after the files are downloaded
* [files](#specifying-files)
  + The files to download

#### Specifying Files

Each file download contains a collection of files to download. These elements are objects consisting of two fields.

* `source` 
  + The URL to download this file from
  + Currently, these are used as-is, so they should be http encoded in the configuration
  + Public URL values only, no authentication is currently offered
* `target` 
  + The filename to give to the downloaded file.
  + This will be combined with `base_dir` 

### Installing Snap Packages

**Note:** Currently, `spinup` does not check for the existence of `snapd` on your system and/or install it. This will be coming in the future, but until then please ensure it is in your package lists.

The `snaps` section of the configuration allows you to specify snap packages to install. There are two different subsections which are similar to how packages work.

* `standard_snaps` 
  + This is just a flat list of snap names
  + These snaps will all be installed from the `stable` channel
  + Snaps requiring `classic` confinement cannot be installed via this list
* [alternate_snaps](#snaps-with-special-requirements)
  + These are snaps that require different channels and/or confinement
  + This is an optional list of objects allowing you to specify individual settings

#### Snaps with Special Requirements

The objects in the `alternate_snaps` section have the following values:

* `name` 
  + This is the name of the snap to install
* `classic` 
  + true/false for whether to install this snap with classic confinement
* `channel` 
  + An enum value for the channel this snap should come from, allowed values are:
    1. `stable` 
    2. `beta` 
    3. `candidate` 
    4. `edge` 
  + This tool will not attempt to resolve any issues with channels, if the snap does not exist in the specified channel, the operation will fail.

### Custom Commands

For actions that are not yet built-in to the program, custom commands can be leveraged to run arbitrary shell commands.

Currently, these are not accepted in the main configuration as their full design has not been completely fleshed out. However they are currently in use during file download operations.

Custom commands have the following properties:

* `command` 
  + This is the name of the command to run
* `args` 
  + An optional collection of strings that should be passed to `command` 
* `needs_root` 
  + Indicates whether `spinup` should run this command via `sudo` 

## Future Additions

As `spinup` is still not finished, there are a lot of extra features that I'd like to add. This includes:

* Flatpak support
* Custom commands in config
* Direct integration with package manager libs instead of using shell processes
* Better handling of missing packages, and other errors
* More testing, including integration testing

