CURRENT_DIR := $(shell pwd)
SOURCES=$(sort $(wildcard ./src/*.rs ./src/**/*.rs))
SHELL := /bin/bash
OS_NAME=$(shell uname | tr '[:upper:]' '[:lower:]')

.PHONY: init
init:
	rustup target add aarch64-apple-ios x86_64-apple-ios
	rustup target add aarch64-apple-darwin x86_64-apple-darwin
	rustup target add aarch64-apple-ios-sim
	#rustup target add armv7-apple-ios armv7s-apple-ios i386-apple-ios ## deprecated
	rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
	rustup target add aarch64-unknown-linux-gnu x86_64-unknown-linux-gnu
	@if [ $$(uname) == "Darwin" ] ; then cargo install cargo-lipo ; fi
	cargo install cbindgen
	cargo install cargo-ndk

all: swift-ios swift-darwin swift kotlin bindings-android

ios-universal: $(SOURCES)		
	mkdir -p ./target/ios-universal/release
	mkdir -p ./target/ios-universal-sim/release
	cargo build --release --target aarch64-apple-ios ;\
	cargo build --release --target x86_64-apple-ios ;\
	cargo build --release --target aarch64-apple-ios-sim ;\
	# build universal lib for arm device and x86 sim
	lipo -create -output ./target/ios-universal/release/libotsffi.a ./target/aarch64-apple-ios/release/libotsffi.a ./target/x86_64-apple-ios/release/libotsffi.a
	# build universal lib for arm sim and x86 sim
	lipo -create -output ./target/ios-universal-sim/release/libotsffi.a ./target/aarch64-apple-ios-sim/release/libotsffi.a ./target/x86_64-apple-ios/release/libotsffi.a

darwin-universal: $(SOURCES)
	mkdir -p ./target/darwin-universal/release
	cargo lipo --release --targets aarch64-apple-darwin
	cargo lipo --release --targets x86_64-apple-darwin
	lipo -create -output ./target/darwin-universal/release/libotsffi.dylib ./target/aarch64-apple-darwin/release/libotsffi.dylib ./target/x86_64-apple-darwin/release/libotsffi.dylib
	lipo -create -output ./target/darwin-universal/release/libotsffi.a ./target/aarch64-apple-darwin/release/libotsffi.a ./target/x86_64-apple-darwin/release/libotsffi.a

swift-ios: ios-universal
	cargo run --features=uniffi/cli --bin uniffi-bindgen generate src/ots.udl -l swift -o ffi/swift-ios
	cp ./target/ios-universal/release/libotsffi.a ffi/swift-ios
	cd ffi/swift-ios && "swiftc" "-emit-module" "-module-name" "otsffi"  "-Xcc" "-fmodule-map-file=$(CURRENT_DIR)/ffi/swift-ios/otsFFI.modulemap" "-I" "."  "-L" "." "-lotsffi" ots.swift

swift-darwin: darwin-universal
	cargo run --features=uniffi/cli --bin uniffi-bindgen generate src/ots.udl -l swift -o ffi/swift-darwin
	cp ./target/darwin-universal/release/libotsffi.dylib ffi/swift-darwin
	cd ffi/swift-darwin && "swiftc" "-emit-module" "-module-name" "otsffi"  "-Xcc" "-fmodule-map-file=$(CURRENT_DIR)/ffi/swift-darwin/otsFFI.modulemap" "-I" "."  "-L" "." "-lotsffi" ots.swift

swift: ios-universal darwin-universal
	mkdir -p swift/Sources/Opentimestamps
	cargo run --features=uniffi/cli --bin uniffi-bindgen generate src/ots.udl --no-format --language swift --out-dir swift/Sources/Opentimestamps
	mv swift/Sources/Opentimestamps/ots.swift swift/Sources/Opentimestamps/otsFFI.swift
	cp swift/Sources/Opentimestamps/otsFFI.h swift/otsFFI.xcframework/ios-arm64/otsFFI.framework/Headers
	cp swift/Sources/Opentimestamps/otsFFI.h swift/otsFFI.xcframework/ios-arm64_x86_64-simulator/otsFFI.framework/Headers
	cp swift/Sources/Opentimestamps/otsFFI.h swift/otsFFI.xcframework/macos-arm64_x86_64/otsFFI.framework/Headers
	cp ./target/aarch64-apple-ios/release/libotsffi.a swift/otsFFI.xcframework/ios-arm64/otsFFI.framework/otsFFI
	cp ./target/ios-universal-sim/release/libotsffi.a swift/otsFFI.xcframework/ios-arm64_x86_64-simulator/otsFFI.framework/otsFFI
	cp ./target/darwin-universal/release/libotsffi.a swift/otsFFI.xcframework/macos-arm64_x86_64/otsFFI.framework/otsFFI

kotlin: android
	cargo run --features=uniffi/cli --bin uniffi-bindgen generate src/ots.udl --language kotlin -o ffi/kotlin

android: aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android

aarch64-linux-android: $(SOURCES) ndk-home
	cargo ndk -t aarch64-linux-android -o ffi/kotlin/jniLibs build --release	
	cp -a $(ANDROID_NDK_HOME)/toolchains/llvm/prebuilt/$(OS_NAME)-x86_64/sysroot/usr/lib/aarch64-linux-android/libc++_shared.so ffi/kotlin/jniLibs/arm64-v8a/

armv7-linux-androideabi: $(SOURCES) ndk-home
	cargo ndk -t armv7-linux-androideabi -o ffi/kotlin/jniLibs build --release
	cp -a $(ANDROID_NDK_HOME)/toolchains/llvm/prebuilt/$(OS_NAME)-x86_64/sysroot/usr/lib/arm-linux-androideabi/libc++_shared.so ffi/kotlin/jniLibs/armeabi-v7a/

i686-linux-android: $(SOURCES) ndk-home
	cargo ndk -t i686-linux-android -o ffi/kotlin/jniLibs build --release
	cp -a $(ANDROID_NDK_HOME)/toolchains/llvm/prebuilt/$(OS_NAME)-x86_64/sysroot/usr/lib/i686-linux-android/libc++_shared.so ffi/kotlin/jniLibs/x86/

x86_64-linux-android: $(SOURCES) ndk-home
	cargo ndk -t x86_64-linux-android -o ffi/kotlin/jniLibs build --release
	cp -a $(ANDROID_NDK_HOME)/toolchains/llvm/prebuilt/$(OS_NAME)-x86_64/sysroot/usr/lib/x86_64-linux-android/libc++_shared.so ffi/kotlin/jniLibs/x86_64/

bindings-android: kotlin
	cp -r ffi/kotlin/jniLibs bindings-android/lib/src/main
	cp -r ffi/kotlin/org bindings-android/lib/src/main/kotlin/
	cd bindings-android && ./gradlew assemble
	mkdir -p ffi/android
	cp bindings-android/lib/build/outputs/aar/lib-release.aar ffi/android

bindings-kotlin-multiplatform: ios-universal kotlin
	mkdir -p bindings-kotlin-multiplatform/ots-kmp/src/androidMain
	cp -r ffi/kotlin/jniLibs/ bindings-kotlin-multiplatform/ots-kmp/src/androidMain/jniLibs/
	cp -r ffi/kmm/* bindings-kotlin-multiplatform/ots-kmp/src/

	cp -r bindings-kotlin-multiplatform/ots-kmp/src/jvmMain/kotlin bindings-kotlin-multiplatform/ots-kmp/src/androidMain/

	mkdir -p bindings-kotlin-multiplatform/ots-kmp/src/libs/ios-arm64/
	mkdir -p bindings-kotlin-multiplatform/ots-kmp/src/libs/ios-simulator-arm64/
	mkdir -p bindings-kotlin-multiplatform/ots-kmp/src/libs/ios-simulator-x64/

	cp ./target/aarch64-apple-ios/release/libots.a bindings-kotlin-multiplatform/ots-kmp/src/libs/ios-arm64/
	cp ./target/aarch64-apple-ios-sim/release/libots.a bindings-kotlin-multiplatform/ots-kmp/src/libs/ios-simulator-arm64/
	cp ./target/x86_64-apple-ios/release/libots.a bindings-kotlin-multiplatform/ots-kmp/src/libs/ios-simulator-x64/
	cd bindings-kotlin-multiplatform && ./gradlew :ots-kmp:assemble

.PHONY: ndk-home
ndk-home:
	@if [ ! -d "${ANDROID_NDK_HOME}" ] ; then \
		echo "Error: Please, set the ANDROID_NDK_HOME env variable to point to your NDK folder" ; \
		exit 1 ; \
	fi

## clean:
.PHONY: clean
clean:
	cargo clean
	rm -rf ffi
	rm -rf kmm