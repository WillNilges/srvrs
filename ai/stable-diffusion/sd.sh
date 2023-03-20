#!/bin/bash
set -e
SD_PROMPT=$1
if [[ -z "$SD_PROMPT" ]]; then echo "Need file argument"; exit 1; fi
podman run --rm -it --hooks-dir=/usr/share/containers/oci/hooks.d/ --security-opt=label=disable -v $(dirname $SD_PROMPT):/workdir:Z -e SD2_PROMPT=$(basename $SD_PROMPT) --name="$(whoami)_$(basename $SD_PROMPT)" 'docker.io/willnilges/srvrs-sd:v1.0'
