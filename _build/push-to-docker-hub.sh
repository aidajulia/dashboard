#!/bin/bash

set -e

ACCOUNT=xliiv
PROJECT=dashboard
IMAGE_NAME="$ACCOUNT"/"$PROJECT"
IMAGE_NAME_COMMIT="$IMAGE_NAME":"$TRAVIS_COMMIT"
IMAGE_NAME_TAG="$IMAGE_NAME":"$TRAVIS_TAG"



if [ "$TRAVIS_BRANCH" == "master" ] && [ "$TRAVIS_RUST_VERSION" == "stable" ]
then
    echo "Releasing commit version .."

    docker login -u="$DOCKER_USERNAME" -p="$DOCKER_PASSWORD"
    docker tag "$IMAGE_NAME" "$IMAGE_NAME_COMMIT"
    docker push "$IMAGE_NAME"

    echo docker push "$IMAGE_NAME_COMMIT"
    if [ -n ${TRAVIS_TAG+x} ]
    then
        echo "Releasing tag version .."

        docker tag "$IMAGE_NAME" "$IMAGE_NAME_TAG"
        docker push "$IMAGE_NAME_TAG"

        echo "Released tag version.."
    fi
    echo "Released commit version.."
fi
