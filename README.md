# Cake
![GitHub License](https://img.shields.io/github/license/dinikai/cake)
![Maintenance](https://img.shields.io/maintenance/yes/2026)
![GitHub Tag](https://img.shields.io/github/v/tag/dinikai/cake?label=version)

A simple *rsync*/*git*-like file synchronization daemon & command-line tool.

## Overview
Cake uses the **warps** system. A warp has an *identifier* (unique name) and some directories bound to it. The beauty of this system is that we don't have to care about *where exactly* remote files are stored (and vice versa: remote peer doesn't have to know anything about our files location).

We have to provide a path for each warp *individually* for every peer in a network, and this is where a **configuration file** comes in. It uses YAML and has a pretty simple schema:
```yaml
bind: 0.0.0.0:39746 # Daemon binding address
confirm: false      # Ask for a confirmation for dangerous operations

warps:
  - name: my-project        # An example warp
    path: /home/me/project1 # with a path

  - name: notes             # Another one;
    path: /home/me/notes    # also has a path

aliases:
  - name: laptop              # The "laptop" name is
    host: 192.168.1.107:39746 # bound to this endpoint
```
The configuration file is located at `~/.config/cake.yaml` and will be created at the very first start of the CLI/daemon with default fields.

Cake offers a straightforward command-line syntax to deal with files within a warp:
```bash
# Send local files to the peer "laptop"
cake push laptop

# Ask the "laptop" to send us its own version of files
cake pull laptop

# Print all differences between the local warp and the "laptop" warp
cake diff laptop
```
As you can see, this reminds pushing to a git repo or pulling from it.

We also can manage warps and peers' aliases (which are listed in the config file) via command-line:
```bash
# Warp management
cake warp add notes-warp /at/this/dir
cake warp remove notes-warp

# Alias management
cake alias add i-am-alias 192.168.107.1
cake alias remove i-am-alias
```

## Usage warnings
> [!WARNING]
> Please this article and be thoughtful to consider its content if you're willing to use Cake.

Cake is now at its **very**(!) early and raw stage of development. I **do not** recommend to use Cake with:
* Any production or production-like environment
* Any type of sensitive and/or important data

Cake **does not** have any traffic encryption for now. That's why you don't want to use it with any sensitive data.

Some day Cake will cross this unsafe border and will be suitable enough to be used in these important cases listed above.

*But* of course, you can use Cake within your own local network (as I personally do while developing Cake itself, for instance) or with any kind of unimportant and non-sensitive files.

## Building
To build Cake you will need the **Cargo** tool installed on your machine.

Clone this repo, `cd` into its directory and build it with Cargo:
```bash
git clone https://github.com/dinikai/cake.git
cd cake
cargo build --release
```
Executables will be placed in the `target/release` directory:
* `cake` for command-line tool
* `caked` for daemon

You can later assign a *systemd* (or any other init system) service to the `caked` executable.

## Roadmap
* [ ] Get rid of potential *quiet* data corruption and writing/reading while transmitting it via TCP (by adding checksums or any other type of post-check)
* [ ] Introduce improved protocol versioning (major & minor)
* [ ] Encrypt network stream with some kind of encryption algorythm
* [ ] Extend the configuration file to give more customization opportunities to user
* [ ] Introduce better config validation
* [ ] Introduce dry-run (*rsync*-like thing)

## Feedback
You can report a bug, request a feature or do anything like that either via [Issues](https://github.com/dinikai/cake/issues) or [Discussions](https://github.com/dinikai/cake/discussions).

I would be very glad to get a feedback!
