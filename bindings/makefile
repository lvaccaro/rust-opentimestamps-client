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

all: swift-ios swift-darwin swift

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
