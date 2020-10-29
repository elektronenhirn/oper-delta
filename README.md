# Scope
oper-delta is branch diffing tool for git repositories managed by google's [git-repo tool](https://source.android.com/setup/develop/repo).

It was developed out of the need to maintain develop branches while main branches continously evolve.

It can calculate the qualitative delta between the current head and one or more given target branches across all repo managed by git-repo.

# Installation

## Ubuntu

Ubuntu releases are available [here](https://github.com/elektronenhirn/oper-delta/releases/latest). After downloading the package which fits your ubuntu version you can install it with

```
sudo apt install ./<path-to-deb-file>
```

## Other Operating Systems

oper-delta is written in rust. You need the rust toolchain installed to be able to use it:

https://www.rust-lang.org/tools/install

Then you simply install _oper-delta_ with:

```
cargo install oper-delta
```

# Usage


## Custom Commands

You can run external executables on the currently selected commit. Running _gitk_ with the key _i_ is one example. You can add more custom commands on your own in oper-delta's config file. The location of the config file depends on your operating system:

- __Mac OS:__  typically at `/Users/<username>/Library/Application Support/oper-delta/config.toml`
- __Ubuntu:__ typically at `/home/<username>/.config/oper-delta/config.toml`

Here we define a custom command to run _git show_ in a new terminal window:

```
# Start gitk whenever 'i' is pressed, the current selected commit
# will be selected in gitk then.
[[custom_command]]
key = "i"
executable = "gitk"
args = ""

# Execute tig in a seperate terminal window
[[custom_command]]
key = "t"
executable = "gnome-terminal"
args = "-- tig --all"

# Open a terminal window in the folder of the selected folder
[[custom_command]]
key = "c"
executable = "gnome-terminal"
args = ""
```

#### Remarks

- The working directory of the new process is set to the directory of the git repository where the selected commit belongs to.
- You cannot run a command line executable in the same terminal as where oper-delta is running, as this would interfer with oper-delta's UI. Wrap your command into a new terminal instance instead (as seen in the example above).
- You cannot override/assign keys which are already built-in (like `j`, `k` and `q`).