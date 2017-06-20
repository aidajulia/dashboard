#!/bin/sh

set -e

# go to repo root
SCRIPT_DIR=$(dirname "$0")
cd $SCRIPT_DIR/..
# TODO checkout branch to temp dir instead of this
#if ! [ -z "$(git status --porcelain)" ]; then echo "Reset repo then run it"; exit 1; fi

RUST_VERSION=${RUST_VERSION:-stable}
echo "rust version:" $RUST_VERSION
# run redis (TODO? running by docker-compose-deps.yml)
REDIS_CID=$(docker run -d redis)
echo "redis id:" $REDIS_CID
REDIS_IP=`docker inspect -f '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}' $REDIS_CID`
echo "redis ip:" $REDIS_IP

docker build \
    -f _build/Dockerfile-test \
    -t dashboard-tested . \
    --no-cache --rm \
    --build-arg REDIS_IP=$REDIS_IP \
    --build-arg RUST_VERSION=$RUST_VERSION

docker stop $REDIS_CID