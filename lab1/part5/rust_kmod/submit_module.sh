#!/usr/bin/env sh

MOD_DIR=~/Yale/CPSC429/cpsc429_labs/lab1/part5/rust_kmod
CURRENT_BRANCH=main

cd $MOD_DIR
git checkout $CURRENT_BRANCH

git commit -a -m "$1"
git pull origin $CURRENT_BRANCH
git push origin $CURRENT_BRANCH

