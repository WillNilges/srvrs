#!/bin/bash
set -e

# Location of the video/audio file
FILE_PATH=$1
if [[ -z "$FILE_PATH" ]]; then echo "Need file as argument"; exit 1; fi

# GPU to run this on from the container's perspective.
# Thanks to CDI, can be hardcoded. 
IN_CONTAINER_DEVICE="cuda:0"

# The GPU that SRVRS will make accessible to the container (different from above)
NVIDIA_GPU="$2" 
# Fallback to GPU 0 if GPU is not provided
if [[ -z "$NVIDIA_GPU" ]]; then NVIDIA_GPU="0"; fi

podman run --rm -it \
	--device nvidia.com/gpu=$NVIDIA_GPU \
	-v $(dirname $FILE_PATH):/workdir:Z \
	-e WHISPER_VIDEO_PATH=$(basename $FILE_PATH) \
	-e WHISPER_DEVICE=$IN_CONTAINER_DEVICE \
	--name="$(whoami)_$(basename $FILE_PATH)" \
	"srvrs-whisper"
