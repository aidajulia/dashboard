#!/bin/sh
set -e

# go to repo root
SCRIPT_DIR=$(dirname "$0")
cd $SCRIPT_DIR/..

./_build/_run_test.sh
./_build/_run_integration.sh