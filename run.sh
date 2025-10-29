#!/bin/bash

pushd /home/ec2-user/lobsters_clone
source .env
./target/debug/straight_line
popd
