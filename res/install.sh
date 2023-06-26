#!/bin/bash

set -e

APP=srvrs
USER=srvrs
GROUP=service

# Directories to create/modify
BASE=/var/srvrs
SCRIPTS="$BASE/scripts"

cargo build --release

# sudo useradd $APP | echo "User already added" # We're doing the user through SSSDeez now, because you can't pull SIDs for local users and network users at the same time. GAH!

sudo rm -rf $BASE
sudo mkdir -p $BASE $SCRIPTS

# Install script
sudo cp ai/whisper/whisper.sh ai/stable-diffusion/sd.sh $SCRIPTS
sudo chown -R $USER:$GROUP $BASE 

# Install systemd service and binary
sudo cp res/srvrs.yaml /etc/
sudo cp res/srvrs.service /etc/systemd/system/
sudo cp res/srvrs-distributor.service /etc/systemd/system/
sudo install target/release/$APP /usr/local/sbin/$APP 
sudo install target/release/$APP-distributor /usr/local/sbin/$APP-distributor

sudo /usr/local/sbin/srvrs setup -c /etc/srvrs.yaml 

# Switch user to srvrs and build the containers
build_containers() {
	set -e # Shit and die if something breaks
	echo "Building containers..."
	cd $BASE/ai/
	for dir in ./*; do
		cd $dir
		echo "Found AI: $(pwd)"
		podman build . --tag srvrs-$(basename $dir)
		cd ..
	done
	exit
	echo "Finished building containers"
}

sudo cp -r ai $BASE/
sudo su -c "export BASE=$BASE; $(declare -f build_containers); build_containers" \
	-s /bin/bash $USER

# Launch srvrs!
sudo systemctl enable srvrs
sudo systemctl enable srvrs-distributor

sudo systemctl restart srvrs
sudo systemctl restart srvrs-distributor

echo "Installed $APP."
