#!/bin/bash

GIT_COMMIT=$(git rev-parse HEAD) cargo build --release

