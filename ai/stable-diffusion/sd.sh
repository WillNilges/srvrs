#!/bin/bash
set -e
SD_PROMPT=$1
if [[ -z "$SD_PROMPT" ]]; then echo "Need file argument"; exit 1; fi
SD_DEVICE="cuda:$2" # GPU to run this on (example "cuda:1")

NVIDIA_GPU="$2" # Ditto, but of course everything has to be different and unique (example gpu1)
if [[ -z "$2" ]]; then SD_DEVICE="cuda:0"; NVIDIA_GPU="0"; fi # Fallback to GPU 0 if GPU is not provided

SD_OUTPUT="/workdir/output.png"

echo "Prompt dir: $(dirname $SD_PROMPT)"
echo "Output: $SD_OUTPUT"

echo "chom"

podman run --rm -it				  \
	--device nvidia.com/gpu=$NVIDIA_GPU	  \
	-v $(dirname $SD_PROMPT):/workdir:Z	  \
	-e SD2_PROMPT=$(basename $SD_PROMPT)	  \
	-e SD2_DEVICE=$SD_DEVICE		  \
	-e SD2_OUTPUT=$SD_OUTPUT		  \
	--name="$(whoami)_$(basename $SD_PROMPT)" \
	"srvrs-stable-diffusion"
