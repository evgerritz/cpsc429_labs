#!/bin/bash

MODULE_DIR=~/cpsc429_labs/lab1/rust_kmod
DRIVER_MODULE=rust_mymem.ko
TEST_MODULE=mymem_test.ko
CURRENT_BRANCH=1_p5  #change this to the name of your part 5 git branch

cd $MODULE_DIR
git pull origin $CURRENT_BRANCH
cp -f $DRIVER_MODULE_NAME ~/
cp -f $TEST_MODULE_NAME ~/

cd ~
#sudo insmod $DRIVER_MODULE
#sudo insmod $TEST_MODULE
#./test 
#sudo rmmod $DRIVER_MODULE
#sudo rmmod $TEST_MODULE
