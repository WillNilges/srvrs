#!/bin/bash
set -e

WC_PATH=$1 # Location of the video/audio file
if [[ -z "$WC_PATH" ]]; then echo "Need file as argument"; exit 1; fi

WC_DEVICE="cuda:$2" # GPU to run this on (example "cuda:1")
if [[ -z "$2" ]]; then WC_DEVICE="cuda:0"; fi # Fallback to GPU 0 if GPU is not provided

NVIDIA_GPU="$2" # Ditto, but of course everything has to be different and unique (example gpu1)
if [[ -z "$2" ]]; then SD_DEVICE="cuda:0"; NVIDIA_GPU="0"; fi # Fallback to GPU 0 if GPU is not provided

podman run --rm -it  --device nvidia.com/gpu=$NVIDIA_GPU -v $(dirname $WC_PATH):/workdir:Z -e WHISPER_VIDEO_PATH=$(basename $WC_PATH) -e WHISPER_DEVICE=$WC_DEVICE --name="$(whoami)_$(basename $WC_PATH)" "docker.io/willnilges/webcaptions:v1.0"
