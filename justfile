default:
    just --list

init:
	rustup target add aarch64-apple-ios x86_64-apple-ios
	rustup target add aarch64-apple-darwin x86_64-apple-darwin
	rustup target add aarch64-apple-ios-sim
	#rustup target add armv7-apple-ios armv7s-apple-ios i386-apple-ios ## deprecated
	rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
	rustup target add aarch64-unknown-linux-gnu x86_64-unknown-linux-gnu
	cargo install cargo-lipo
	cargo install cbindgen
	cargo install cargo-ndk

all: swift-ios swift-darwin swift kotlin bindings-android

ios-universal:
	mkdir -p ./target/ios-universal/release
	mkdir -p ./target/ios-universal-sim/release
	cargo build -p ots_bindings --release --target aarch64-apple-ios ;\
	cargo build -p ots_bindings --release --target x86_64-apple-ios ;\
	cargo build -p ots_bindings --release --target aarch64-apple-ios-sim ;\
	# build universal lib for arm device and x86 sim
	lipo -create -output ./target/ios-universal/release/libotsffi.a ./target/aarch64-apple-ios/release/libotsffi.a ./target/x86_64-apple-ios/release/libotsffi.a
	# build universal lib for arm sim and x86 sim
	lipo -create -output ./target/ios-universal-sim/release/libotsffi.a ./target/aarch64-apple-ios-sim/release/libotsffi.a ./target/x86_64-apple-ios/release/libotsffi.a

darwin-universal:
	mkdir -p ./target/darwin-universal/release
	cargo lipo --release --targets aarch64-apple-darwin
	cargo lipo --release --targets x86_64-apple-darwin
	lipo -create -output ./target/darwin-universal/release/libotsffi.dylib ./target/aarch64-apple-darwin/release/libotsffi.dylib ./target/x86_64-apple-darwin/release/libotsffi.dylib
	lipo -create -output ./target/darwin-universal/release/libotsffi.a ./target/aarch64-apple-darwin/release/libotsffi.a ./target/x86_64-apple-darwin/release/libotsffi.a

swift-ios: ios-universal
	cargo run -p ots_bindings --features=uniffi/cli --bin uniffi-bindgen generate ots_bindings/src/ots.udl -l swift -o ots_bindings/ffi/swift-ios
	cp ./target/ios-universal/release/libotsffi.a ots_bindings/ffi/swift-ios
	cd ots_bindings/ffi/swift-ios && "swiftc" "-emit-module" "-module-name" "otsffi"  "-Xcc" -fmodule-map-file=$(pwd)/otsFFI.modulemap "-I" "."  "-L" "." "-lotsffi" ots.swift

swift-darwin: darwin-universal
	cargo run -p ots_bindings --features=uniffi/cli --bin uniffi-bindgen generate ots_bindings/src/ots.udl -l swift -o ots_bindings/ffi/swift-darwin
	cp ./target/darwin-universal/release/libotsffi.dylib ots_bindings/ffi/swift-darwin
	cd ots_bindings/ffi/swift-darwin && "swiftc" "-emit-module" "-module-name" "otsffi"  "-Xcc" -fmodule-map-file=$(pwd)/otsFFI.modulemap "-I" "."  "-L" "." "-lotsffi" ots.swift

swift: ios-universal darwin-universal
	mkdir -p swift/Sources/Opentimestamps
	cargo run -p ots_bindings --features=uniffi/cli --bin uniffi-bindgen generate ots_bindings/src/ots.udl --no-format --language swift --out-dir ots_bindings/swift/Sources/Opentimestamps
	mv ots_bindings/swift/Sources/Opentimestamps/ots.swift ots_bindings/swift/Sources/Opentimestamps/otsFFI.swift
	cp ots_bindings/swift/Sources/Opentimestamps/otsFFI.h ots_bindings/swift/otsFFI.xcframework/ios-arm64/otsFFI.framework/Headers
	cp ots_bindings/swift/Sources/Opentimestamps/otsFFI.h ots_bindings/swift/otsFFI.xcframework/ios-arm64_x86_64-simulator/otsFFI.framework/Headers
	cp ots_bindings/swift/Sources/Opentimestamps/otsFFI.h ots_bindings/swift/otsFFI.xcframework/macos-arm64_x86_64/otsFFI.framework/Headers
	cp ./target/aarch64-apple-ios/release/libotsffi.a ots_bindings/swift/otsFFI.xcframework/ios-arm64/otsFFI.framework/otsFFI
	cp ./target/ios-universal-sim/release/libotsffi.a ots_bindings/swift/otsFFI.xcframework/ios-arm64_x86_64-simulator/otsFFI.framework/otsFFI
	cp ./target/darwin-universal/release/libotsffi.a ots_bindings/swift/otsFFI.xcframework/macos-arm64_x86_64/otsFFI.framework/otsFFI

kotlin: android
	cargo run -p ots_bindings --features=uniffi/cli --bin uniffi-bindgen generate ots_bindings/src/ots.udl --language kotlin -o ots_bindings/ffi/kotlin

android: aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android

aarch64-linux-android:
	cargo ndk -t aarch64-linux-android -o ots_bindings/ffi/kotlin/jniLibs build -p ots_bindings --release

armv7-linux-androideabi:
	cargo ndk -t armv7-linux-androideabi -o ots_bindings/ffi/kotlin/jniLibs build -p ots_bindings --release

i686-linux-android:
	cargo ndk -t i686-linux-android -o ots_bindings/ffi/kotlin/jniLibs build -p ots_bindings --release

x86_64-linux-android:
	cargo ndk -t x86_64-linux-android -o ots_bindings/ffi/kotlin/jniLibs build -p ots_bindings --release

bindings-android: kotlin
	cp -r ots_bindings/ffi/kotlin/jniLibs ots_bindings/bindings-android/lib/src/main
	cp -r ots_bindings/ffi/kotlin/org ots_bindings/bindings-android/lib/src/main/kotlin/
	cd ots_bindings/bindings-android && ./gradlew assemble
	mkdir -p ots_bindings/ffi/android
	cp ots_bindings/bindings-android/lib/build/outputs/aar/lib-release.aar ots_bindings/ffi/android

bindings-kotlin-multiplatform: ios-universal kotlin
	mkdir -p ots_bindings/bindings-kotlin-multiplatform/ots-kmp/src/androidMain
	cp -r ots_bindings/ffi/kotlin/jniLibs/ ots_bindings/bindings-kotlin-multiplatform/ots-kmp/src/androidMain/jniLibs/
	cp -r ots_bindings/bindings-kotlin-multiplatform/ots-kmp/src/jvmMain/kotlin ots_bindings/bindings-kotlin-multiplatform/ots-kmp/src/androidMain/

	mkdir -p ots_bindings/bindings-kotlin-multiplatform/ots-kmp/src/libs/ios-arm64/
	mkdir -p ots_bindings/bindings-kotlin-multiplatform/ots-kmp/src/libs/ios-simulator-arm64/
	mkdir -p ots_bindings/bindings-kotlin-multiplatform/ots-kmp/src/libs/ios-simulator-x64/

	cp ./target/aarch64-apple-ios/release/libots.a ots_bindings/bindings-kotlin-multiplatform/ots-kmp/src/libs/ios-arm64/
	cp ./target/aarch64-apple-ios-sim/release/libots.a ots_bindings/bindings-kotlin-multiplatform/ots-kmp/src/libs/ios-simulator-arm64/
	cp ./target/x86_64-apple-ios/release/libots.a ots_bindings/bindings-kotlin-multiplatform/ots-kmp/src/libs/ios-simulator-x64/
	cd ots_bindings/bindings-kotlin-multiplatform && ./gradlew :ots-kmp:assemble

clean:
	cargo clean
	rm -rf ffi
	rm -rf kmm
