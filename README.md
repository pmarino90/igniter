# Igniter
A simple process manager written in Rust.  

[![Build Status](https://api.travis-ci.org/pmarino90/igniter.svg?branch=master)](https://travis-ci.org/pmarino90/igniter)
[![crates.io](https://img.shields.io/crates/v/igniter.svg)](https://crates.io/crates/igniter)


**UNDER ACTIVE DEVELOPMENT**

## Description
Igniter was written for fun and profit, the idea behind it is to a simple process manager to be used while developing projects that need some always on dependencies.  
So imagine to be developing a frontend application and you need the API active on your machine and maybe some other dependencies (a database for example), Igniter reads an `.igniterc` file and starts all the processes described into the configuration file.

There are a lot of other process manangers into the wild and most of them use a daemon process which monitors and manages all the child processes. Igniter does not provide a global daemon but instead runs a monitor process for every launched one, so there is a one-to-one relationship bewteen the process to be launched and the one which monitors.

Igniter is in its early stages, under active development and for such reasons is not _yet_ feature complete neither its code is near to be considered good or safe. A list of future development ideas can be found into a further paragraph.  
Suggestion and/or requests are always welcome!

### Supported operating systems
Given its dependencies the supported OSs are the ones which are supported by `nix` (look [here](https://github.com/nix-rust/nix#supported-platforms)), this does not directly means that all the version are tested.

Currently there is no **Windows** support.

## Install
With cargo: 

```bash
cargo install igniter
```
## Usage

With an existing `.igniterc` file into the current working directory issue `igniter` from your terminal and Igniter will spawn all the processes defined into the configuration file.

### .igniterc
Create an `.igniterc`, `TOML` syntax supported, file in the root of your project. It is always needed a `.igniterc` file to start monitoring a process.
```toml
[[process]]
name = "process-1" # Required, the name of the process. Shown in list.
cmd = "process" # The actual command to launch, must be the command without arguments
args = [["-arg", "value"], ["--arg2"]]  # Optional, must be an array of array
max_retries = 10 # Optional, defaults to 0
```


The above block can be repeated any time needed into the same file to describe other processes, just mind to change the name.

### Cli
```
igniter 0.1.0
Paolo Marino <paolomarinos@gmail.com>
A simple process manager

USAGE:
    igniter [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    help       Prints this message or the help of the given subcommand(s)
    list       list active processes
    monitor    [INTERNAL] Monitors the provided command data as JSON
    stop       Stops an already running process given its name
```

## Todo
[x] Process restart on fail  
[x] Refactor  
[] Environment variables  
[] Log Management  
[] Context awareness (Workspaces)  

## Changelog
* v0.1.0  
    * First version
* v0.1.1
    * Code refactor
    * minor bug fixes
