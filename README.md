[![Actions Status](https://github.com/stevepentland/spinup/workflows/Rust/badge.svg)](https://github.com/stevepentland/spinup/actions) [![Dependabot Status](https://api.dependabot.com/badges/status?host=github&repo=stevepentland/spinup&identifier=220822410)](https://dependabot.com)

# Spinup

## Special Note

Spinup is currently beta software

## Background

Spinup is a program designed to make setting up newly installed Linux machines easy and repeatable. There is no complicated software to install, just a single application and a configuration file that can be in json, toml, or yaml format.

It will perform system upgrade, install packages via your package manager, download files from the web, run custom commands, and install snap packages. All actions are defined in a configuration file and passed to the application at runtime.

## Getting Started

Currently, there are no builds provided, but it can be quickly built by cloning this repo and running `cargo build --release` . Spinup will build on current stable rust, with a version of 1.39 or greater.

Once you have the `spinup` binary, you can put it onto a newly installed machine and run it. In the future, builds will be populated in the [Releases](https://github.com/stevepentland/spinup/releases) page.

## Configuration Files

Spinup plays a set of instructions that are provided via a configuration file. Examples of configuration files can be found in the [examples](https://github.com/stevepentland/spinup/tree/master/examples) directory in the project.

