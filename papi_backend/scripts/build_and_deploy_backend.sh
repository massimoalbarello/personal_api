#!/bin/bash

if [ -n "$1" ]; then
    # Ensure that the script executes in its own directory
    cd "$(dirname "$0")"

    # Stop and remove the docker image that might be running
    source ./stop_and_remove_backend.sh $1

    # Switch to the parent directory where the Dockerfile is located
    cd ..

    # Build the docker image
    echo "Building $1 docker image..."
    podman build -t $1 .

    # Deploy the docker container based on the image
    echo "Deploying $1 docker container..."
    podman run -d -p 8443:8443 --name $1 $1

    echo "Server listening on https://localhost:8443"
else
    echo "Need to specify the name of the image to build and run"
fi