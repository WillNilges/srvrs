#!/bin/bash
set -e
SD_PROMPT=$1
if [[ -z "$SD_PROMPT" ]]; then echo "Need file argument"; exit 1; fi
SD_DEVICE="cuda:$2" # GPU to run this on (example "cuda:1")
if [[ -z "$2" ]]; then WC_DEVICE="cuda:0"; fi # Fallback to GPU 0 if GPU is not provided
podman run --rm -it --hooks-dir=/usr/share/containers/oci/hooks.d/ --security-opt=label=disable -v $(dirname $SD_PROMPT):/workdir:Z -e SD2_PROMPT=$(basename $SD_PROMPT) -e SD2_DEVICE=$SD_DEVICE --name="$(whoami)_$(basename $SD_PROMPT)" 'willnilges/srvrs-sd:v1.0'
