#!/bin/bash

set -e

ACCOUNT=xliiv
PROJECT=dashboard
IMAGE_NAME="$ACCOUNT"/"$PROJECT"


if [ "$TRAVIS_BRANCH" == "master" ] && [ "$TRAVIS_RUST_VERSION" == "stable" ]
then
    echo "Releasing commit version .."

    IMAGE_NAME_COMMIT="$IMAGE_NAME":"$TRAVIS_COMMIT"
    docker login -u="$DOCKER_USERNAME" -p="$DOCKER_PASSWORD"
    docker tag "$IMAGE_NAME" "$IMAGE_NAME_COMMIT"
    docker push "$IMAGE_NAME"

    echo "Released commit version: $IMAGE_NAME_COMMIT"
fi

if [ -n "$TRAVIS_TAG" ] && [ "$TRAVIS_RUST_VERSION" == "stable" ]
then
    echo "Releasing tag version .."

    IMAGE_NAME_TAG="$IMAGE_NAME":"$TRAVIS_TAG"
    docker login -u="$DOCKER_USERNAME" -p="$DOCKER_PASSWORD"
    docker tag "$IMAGE_NAME" "$IMAGE_NAME_TAG"
    docker push "$IMAGE_NAME_TAG"

    echo "Released tag version: $IMAGE_NAME_TAG"
fi
