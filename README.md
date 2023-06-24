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

**watch** — Used by the daemon to watch a directory for new work to do, and execute a command on that file.

**status** — Get a brief status update on what SRVRS is doing. Will tell you the timecode that it is currently busy with, error messages, or if it doesn't have anything to do, it will say, "Idle."

**services** — Lists the services that SRVRS offers. Right now, it's just whisper lol.

**queue** — Prints the number of files in the work directory.
