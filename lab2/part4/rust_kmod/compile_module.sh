#!/bin/bash

SHARED_SAMPLES=~/cpsc429_labs/lab2/part4/rust_kmod
BUILD_SAMPLES=~/linux/samples/rust
BUILD_DIR=~/linux/
CURRENT_BRANCH=2_p4

cd $SHARED_SAMPLES
git pull origin $CURRENT_BRANCH
cp -f $SHARED_SAMPLES/custom_modules/rust_camera.rs  $BUILD_SAMPLES/custom_modules/

cd $BUILD_DIR/custom_modules/
make camera
#KRUSTFLAGS="--emit=metadata --extern mymem=$BUILD_SAMPLES/librust_mymem.rmeta" make SUBDIRS=$BUILD_SAMPLES modules -j4 #2>&1 | less
cp -f $BUILD_SAMPLES/custom_modules/rust_camera.ko $SHARED_SAMPLES/custom_modules/

cd $SHARED_SAMPLES
git checkout $CURRENT_BRANCH
git add custom_modules/rust_camera.ko
git commit -m "rebuilt rust_camera.ko"
git push origin $CURRENT_BRANCH
