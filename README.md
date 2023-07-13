# SRVRS *[ sur-vers ]*

An experimental platform for performing file-in, file-out work on a server.

Place a file in a specified directory, and a pre-determined process will launch using your provided file as input. After a time, the output of said process will appear in your home directory. Using this program, it couldn't be simpler to provide your users with access to AI, rendering, and other complicated workflows.

<div id="badges">
<img src="https://forthebadge.com/images/badges/built-with-science.svg" alt="C badge" height="30px"/>
<img src="https://forthebadge.com/images/badges/made-with-out-pants.svg" alt="C badge" height="30px"/>
<img src="https://forthebadge.com/images/badges/made-with-rust.svg" alt="C badge" height="30px"/>
</div>

## Examples

### Whisper

Proof of concept will run a Whisper container to caption a video:

`scp video.mov wilnil@jet.csh.rit.edu:/var/srvrs/whisper/`

When captioning is finished, check your `/scratch` directory for output.

### Stable Diffusion 2.0

You can also ask it to generate an image using Stable Diffusion 2.0:

`echo "strawberry sushi" > /tmp/prompt.txt && scp /tmp/prompt.txt wilnil@jet.csh.rit.edu:/var/srvrs/stable-diffusion`

On my machine, with a K80, it will take about 5 minutes per image.

You can check the status by running `srvrs status` on Jet. When it is finished, it will output the file to your `/scratch` directory.

## Structure

TODO lol

SRVRS is a daemon that runs on a machine to allow people to perform operations on files that they place in a certain directory. This is useful because it provides users with a zero-configuration way to access services that may be complicated or tedious to setup and maintain on their own (such as whisper or stable diffusion), and provides a reproducable way to maintain these services for the sysadmin.

SRVRS provides access to **Activities**, which, simply put, is a _thing that SRVRS can do for you_. Each Activity is composed of a few things:
- A Dockerfile, to describe the environment in which this activity should run. You can use this to specify an image, install dependencies, download files, etc.
- A script to describe how to launch the container, and how to pass arguments to it.

One weakness of SRVRS currently is that it has no way to customize arguments. Each activity pretty much only has the option of passing in a file. Granted, that file could have configuration in it. There's technically nothing stopping you from creating an activity that takes in a zip full of YAML and other stuff.

Technically, you could skip the Dockerfile and use the script to execute arbitrary code baremetal. **This is not recommended.** SRVRS is supposed to allow you to compartmentalize and make your services reproducable.

SRVRS is split into two daemons: The main one, and a distributor. The main one is responsible for managing activies, watching for new input files, and spinning up containers. The distributor is only responsible for moving the finished work to its final destination, usually the homedir of a user. The reason for this is to prevent SRVRS itself from running as root, since it technically allows for arbitrary code execution.

Both have a set of flags and options that are used in their configs, systemd services, and at runtime.

**SRVRS**
```
Usage: srvrs <COMMAND>

Commands:
  setup
  watch
  status
  services
  queue
  help      Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

**DISTRIBUTOR**
```
Usage: srvrs-distributor --work-path <WORK_PATH> --destination-base-path <DESTINATION_BASE_PATH>

Options:
  -w, --work-path <WORK_PATH>                          Path where we do our work
  -d, --destination-base-path <DESTINATION_BASE_PATH>  Path where we put the finished product
  -h, --help                                           Print help
  -V, --version                                        Print version
```

## Installation

1. Install prereqs (This assumes you've got your nvidia drivers set up already)
- https://rustup.rs/
- https://podman.io/
- https://nvidia.github.io/libnvidia-container/

2. Clone the project, and navigate to the `res/` directory.
```
git clone https://github.com/willnilges/srvrs
cd srvrs/res
```

3. Inspect the systemd services and the config file. Here is where you ought to make any changes to parameters such as available activies, destinations for users and the like. The defaults should mostly work for you, except for the destination directory. Normal, shared, non-networked machines probably want to use `/home`. **Be sure that the install script, config files, and systemd services all agree on file paths.**

4. Run the install script.
```
./install.sh
```

It will
- Add a local SRVRS user (if necessary)
- Compile the project with Cargo
- Clean up any previous installation's garbage (watch out if you've changed install locations; That might leave behind garbage)
- Set permissions
- Create directories, install config files, install binaries, install activities
- Run first-time setup
- Build activity containers as the local user
- Enable and start the services

## Sub-Commands

**setup** — Used by the installer to do some first-time configuration. **Not for human consumption.**

**watch** — Used by the daemon to watch a directory for new work to do, and execute a command on that file. **Not for human consumption.**

**status** — Get a brief status update on what SRVRS is doing. Will tell you the timecode that it is currently busy with, error messages, or if it doesn't have anything to do, it will say, "Idle."

**queue** — Prints the number of files in the work directory.
