<div align="center">

  # cake

  <img src="logo.png" alt="Logo" width="200">
  <br><br>

  ![GitHub License](https://img.shields.io/github/license/dinikai/cake)
  ![Maintenance](https://img.shields.io/maintenance/yes/2026)
  ![GitHub Tag](https://img.shields.io/github/v/tag/dinikai/cake?label=version)
  <br>
  A simple **rsync**/**git**-like file synchronization daemon & command-line tool.
</div>

## Overview
Cake uses the **warps** system. A warp has an *identifier* (unique name) and some directories bound to it. The beauty of this system is that we don't have to care about *where exactly* remote files are stored (and vice versa: remote peer doesn't have to know anything about our files location).

Cake offers a straightforward command-line syntax to deal with files within a warp:
```bash
# Send local files to the peer "laptop"
$ cake push laptop

# Ask the "laptop" to send us its own version of files
$ cake pull laptop

# Print all differences between the local warp and the "laptop" warp
$ cake diff laptop
```
As you can see, Cake warp reminds a Git repo without history.

## Building
To build Cake you need the **Cargo** tool installed on your machine.

Clone this repo, `cd` into its directory and build it with Cargo:
```bash
git clone https://github.com/dinikai/cake.git
cd cake
cargo build --release
```
Executables will be placed in the `target/release` directory:
* `cake`: the command-line tool
* `caked`: the daemon (server)

You can later assign a *systemd* (or any other init system) service to the `caked` executable.

## The guide
To get the detailed step-by-step guide, you may want to visit the [Wiki](https://github.com/dinikai/cake/wiki). It will explain you most things you would want to know about Cake.

## Usage warnings
> [!WARNING]
> Please this article and be thoughtful to consider its content if you're willing to use Cake.

Cake is now at its **very**(!) early and raw stage of development. I **do not** recommend to use Cake with:
* Any production or production-like environment
* Any type of sensitive and/or important data

Cake **does not** have any traffic encryption for now. That's why you don't want to use it with any sensitive data.

Some day Cake will cross this unsafe border and will be suitable enough to be used in these important cases listed above.

*But* of course, you can use Cake within your own local network (as I personally do while developing Cake itself, for instance) or with any kind of unimportant and non-sensitive files.

## Roadmap
### Fundamental goals
The next goals **must** be completed before Cake can be safely used in the dangerous cases [listed above](#usage-warnings):
* [x] **Introduce the token authentication system**
* [ ] **Encrypt network streams with some secure encryption algorithm**

### Non-fundamental goals
* [x] Begin using of the `tokio` runtime
* [x] Make checksum calculation process asynchronous
* [ ] Get rid of the potential *quiet* data corruption while transmitting it via TCP (by adding checksums or any other type of post-check)
* [ ] Introduce the improved protocol versioning (major & minor)

### User experience improvement goals
* [ ] Extend the configuration file to give more customization opportunities to a user
* [ ] Introduce the better config validation
* [ ] Introduce dry-run (*rsync*-like thing)

## Feedback
You can report a bug, request a feature or do anything like that either via [Issues](https://github.com/dinikai/cake/issues) or [Discussions](https://github.com/dinikai/cake/discussions).

I would be very glad to get a feedback!
