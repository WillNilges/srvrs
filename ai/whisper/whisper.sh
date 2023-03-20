#!/bin/bash
set -e
WC_PATH=$1 # Location of the video/audio file
WC_DEVICE="cuda:$2" # GPU to run this on (example "cuda:1")
if [[ -z "$2" ]]; then WC_DEVICE="cuda:0"; fi # Fallback to GPU 0 if GPU is not provided
if [[ -z "$WC_PATH" ]]; then echo "Need file as argument"; exit 1; fi
podman run --rm -it --hooks-dir=/usr/share/containers/oci/hooks.d/ --security-opt=label=disable -v $(dirname $WC_PATH):/workdir:Z -e WHISPER_VIDEO_PATH=$(basename $WC_PATH) -e WHISPER_DEVICE=$WC_DEVICE --name="$(whoami)_$(basename $WC_PATH)" "docker.io/willnilges/webcaptions:v1.0"
