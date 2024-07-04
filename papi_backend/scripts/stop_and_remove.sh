#!/bin/bash

if [ -n "$1" ]; then
    echo "Stopping $1 docker container..."
    podman stop $1

    echo "Removing $1 docker container..."
    podman rm $1
else
    echo "Need to specify the name of the image to stop and remove"
fi