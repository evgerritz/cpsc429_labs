#!/bin/bash

MODULE_DIR=~/cpsc429_labs/lab2/part4/rust_kmod/custom_modules
MODULE=rust_camera
CURRENT_BRANCH=2_p4  #change this to the name of your part 5 git branch

cd $MODULE_DIR
git pull origin $CURRENT_BRANCH
lsmod | grep rust_camera && sudo rmmod $MODULE
sudo insmod $MODULE.ko
