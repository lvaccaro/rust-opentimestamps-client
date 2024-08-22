#!/bin/sh
source <(cargo ndk-env)
export NDK_SYSROOT_LIBS_PATH=$CARGO_NDK_SYSROOT_LIBS_PATH
export OS_NAME=$(uname -o | tr  '[:upper:]' '[:lower:]')
#NDK_SYSROOT_LIBS_PATH=${ANDROID_NDK_HOME}/toolchains/llvm/prebuilt/${OS_NAME}-x86_64/sysroot/usr/lib/aarch64-linux-android/
