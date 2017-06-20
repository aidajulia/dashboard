#!/bin/sh

set -e

# docker image where app was tested and succeed
TESTED_IMAGE=dashboard-tested
# host path where app is extracted from tested-image
APP_PATH=_build/dashboard
BOWER_COMPONENTS=_build/bower_components

# go to repo root
SCRIPT_DIR=$(dirname "$0")
cd $SCRIPT_DIR/..
# TODO checkout branch to temp dir instead of this
#if ! [ -z "$(git status --porcelain)" ]; then echo "Reset repo then run it"; exit 1; fi

docker create --name running-dashboard-tested dashboard-tested
docker cp running-dashboard-tested:/home/rust/src/target/x86_64-unknown-linux-musl/release/dashboard $APP_PATH
docker cp running-dashboard-tested:/home/rust/src/src/static/bower_components $BOWER_COMPONENTS
docker rm -f running-dashboard-tested

docker build \
    -f _build/Dockerfile-integration \
    -t xliiv/dashboard . \
    --build-arg APP_PATH=$APP_PATH \
    --build-arg BOWER_COMPONENTS=$BOWER_COMPONENTS \
    --no-cache

rm -rf $APP_PATH $BOWER_COMPONENTS