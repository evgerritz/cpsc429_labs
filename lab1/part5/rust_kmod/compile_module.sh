#!/bin/bash

SHARED_SAMPLES=~/cpsc429_labs/lab1/rust_kmod
BUILD_SAMPLES=~/linux3/samples/rust
BUILD_DIR=~/linux3/
CURRENT_BRANCH=1_p5

cd $SHARED_SAMPLES
git pull origin $CURRENT_BRANCH
cp -f $SHARED_SAMPLES/rust_mymem.rs  $BUILD_SAMPLES/
cp -f $SHARED_SAMPLES/mymem_test.rs  $BUILD_SAMPLES/
cp -f $SHARED_SAMPLES/Makefile  $BUILD_SAMPLES/

cd $BUILD_DIR
KRUSTFLAGS="--emit=metadata --extern mymem=$BUILD_SAMPLES/librust_mymem.rmeta" make SUBDIRS=$BUILD_SAMPLES modules -j4 #2>&1 | less
cp -f $BUILD_SAMPLES/rust_mymem.ko $SHARED_SAMPLES/
cp -f $BUILD_SAMPLES/mymem_test.ko $SHARED_SAMPLES/

cd $SHARED_SAMPLES
git checkout $CURRENT_BRANCH
git add rust_mymem.ko
git add mymem_test.ko
git commit -m "rebuilt mymem.ko and mymem_test.ko"
git push origin $CURRENT_BRANCH
