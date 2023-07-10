# SRVRS *[ sur-vers ]*

An experimental platform for performing file-in-file-out work on a server.

Place a file in a specified directory, and a command/script/whatever will run, ideally with your provided file as input.

## How to use

### Whisper

Proof of concept will run a Whisper container to caption a video:

`scp video.mov wilnil@jet.csh.rit.edu:/var/srvrs/whisper/`

When captioning is finished, check your `/scratch` directory for output.

### Stable Diffusion 2.0

You can also ask it to generate an image using Stable Diffusion 2.0:

`echo "strawberry sushi" > /tmp/prompt.txt && scp /tmp/prompt.txt wilnil@jet.csh.rit.edu:/var/srvrs/stable-diffusion`

That'll take about 5 minutes per image.

SRVRS is currently not multithreaded, as it seems like a bad idea to have it able to use more than one GPU at a time. It also isn't yet smart enough to pick a free GPU, or to wait until a GPU is free.

 It will take some time. You can check the status by running `srvrs status` on Jet. When it is finished, it will output the file to your `/scratch` directory.

## Commands

**watch** — Used by the daemon to watch a directory for new work to do, and execute a command on that file. Not for human consumption.

**status** — Get a brief status update on what SRVRS is doing. Will tell you the timecode that it is currently busy with, error messages, or if it doesn't have anything to do, it will say, "Idle."

~~**services** — Lists the services that SRVRS offers. Right now, it's just whisper lol.~~

**queue** — Prints the number of files in the work directory.

## Structure

TODO lol

SRVRS is a daemon that runs on a machine to allow people to perform operations on files that they place in a certain directory. This is useful because it provides users with a zero-configuration way to access services that may be complicated or tedious to setup and maintain on their own (such as whisper or stable diffusion), and provides a reproducable way to maintain these services for the sysadmin.

SRVRS provides access to **Activities**, which, simply put, is a _thing that SRVRS can do for you_. Each Activity is composed of a few things:
- A Dockerfile, to describe the environment in which this activity should run. You can use this to specify an image, install dependencies, download files, etc.
- A script to describe how to launch the container, and how to pass arguments to it.

One weakness of SRVRS currently is that it has no way to customize arguments. Each activity pretty much only has the option of passing in a file. Granted, that file could have configuration in it. There's technically nothing stopping you from creating an activity that takes in a zip full of YAML and other stuff.

Technically, you could skip the Dockerfile and use the script to execute arbitrary code baremetal. **This is not recommended.** SRVRS is supposed to allow you to compartmentalize and make reproducable your services.
