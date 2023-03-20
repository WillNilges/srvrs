#!/bin/bash

set -e

APP=srvrs

# Directories to create/modify
BASE=/var/srvrs
SCRIPTS="$BASE/scripts"

cargo build --release

sudo useradd $APP | echo "User already added"

sudo rm -rf $BASE
sudo mkdir -p $BASE $SCRIPTS

# Install script
sudo cp ai/whisper/whisper.sh ai/stable-diffusion/sd.sh $SCRIPTS
sudo chown -R $APP:$APP $BASE 

# Install systemd service and binary
sudo cp res/srvrs.yaml /etc/
sudo cp res/srvrs.service /etc/systemd/system/
sudo cp res/srvrs-distributor.service /etc/systemd/system/
sudo install target/release/$APP /usr/local/sbin/$APP 
sudo install target/release/$APP-distributor /usr/local/sbin/$APP-distributor

# Launch srvrs!
sudo systemctl enable srvrs
sudo systemctl enable srvrs-distributor

sudo systemctl restart srvrs
sudo systemctl restart srvrs-distributor

echo "Installed $APP."
