#!/bin/bash
set -e

# Path to prompt file
FILE_PATH=$1
if [[ -z "$FILE_PATH" ]]; then echo "Need file argument"; exit 1; fi

# GPU to run this on from the container's perspective.
# Thanks to CDI, can be hardcoded. 
IN_CONTAINER_DEVICE="cuda:0"

# The GPU that SRVRS will make accessible to the container (different from above)
NVIDIA_GPU="$2" 
# Fallback to GPU 0 if GPU is not provided
if [[ -z "$NVIDIA_GPU" ]]; then NVIDIA_GPU="0"; fi 

SD_OUTPUT="/workdir/output.png"

echo "Prompt dir: $(dirname $FILE_PATH)"
echo "Output: $SD_OUTPUT"

echo "chom"

podman run --rm -it				  \
	--device nvidia.com/gpu=$NVIDIA_GPU	  \
	-v $(dirname $FILE_PATH):/workdir:Z	  \
	-e SD2_PROMPT=$(basename $FILE_PATH)	  \
	-e SD2_DEVICE=$IN_CONTAINER_DEVICE		  \
	-e SD2_OUTPUT=$SD_OUTPUT		  \
	--name="$(whoami)_$(basename $FILE_PATH)" \
	"srvrs-stable-diffusion"
