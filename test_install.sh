#!/bin/bash

set -e

APP=srvrs

# Directories to create/modify
BASE=/tmp/wilnil/srvrs
SCRIPTS="$BASE/scripts"

rm -rf $BASE
mkdir -p $BASE $SCRIPTS

# Install script
cp ai/whisper/whisper.sh ai/stable-diffusion/sd.sh $SCRIPTS
#chown -R $APP:$APP $BASE 

echo "Installed $APP."
