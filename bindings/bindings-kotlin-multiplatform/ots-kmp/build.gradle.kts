plugins {
    kotlin("multiplatform")
    id("com.android.library")
    id("maven-publish")
}

apply(plugin = "kotlinx-atomicfu")

kotlin {
    // Enable the default target hierarchy
    applyDefaultHierarchyTemplate()

    androidTarget {
        compilations.all {
            kotlinOptions {
                jvmTarget = JavaVersion.VERSION_17.majorVersion
            }
        }

        publishLibraryVariants("release")
    }

    jvm {
        compilations.all {
            kotlinOptions.jvmTarget = JavaVersion.VERSION_17.majorVersion
        }
    }

    listOf(
        iosX64(),
        iosArm64(),
        iosSimulatorArm64()
    ).forEach {
        val platform = when (it.targetName) {
            "iosSimulatorArm64" -> "ios_simulator_arm64"
            "iosArm64" -> "ios_arm64"
            "iosX64" -> "ios_x64"
            else -> error("Unsupported target $name")
        }

        it.compilations["main"].cinterops {
            create("otsCInterop") {
                defFile(project.file("src/nativeInterop/cinterop/ots.def"))
                includeDirs(project.file("src/nativeInterop/cinterop/headers/ots"), project.file("src/libs/$platform"))
            }
        }
    }

    sourceSets {
        all {
            languageSettings.apply {
                optIn("kotlinx.cinterop.ExperimentalForeignApi")
            }
        }

        val commonMain by getting {
            dependencies {
                implementation("com.squareup.okio:okio:3.6.0")
                implementation("org.jetbrains.kotlinx:kotlinx-datetime:0.4.1")
            }
        }

        val commonTest by getting {
            dependencies {
                implementation(kotlin("test"))
            }
        }

        val jvmMain by getting {
            dependsOn(commonMain)
            dependencies {
                implementation("net.java.dev.jna:jna:5.13.0")
            }
        }

        val androidMain by getting {
            dependsOn(commonMain)
            dependencies {
                implementation("net.java.dev.jna:jna:5.13.0@aar")
                implementation("org.jetbrains.kotlinx:atomicfu:0.23.1")
            }
        }
    }
}

android {
    namespace = "org.opentimestamps"
    compileSdk = 33

    defaultConfig {
        minSdk = 21
        consumerProguardFiles("consumer-rules.pro")
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
}

val libraryVersion: String by project

group = "org.opentimestamps"
version = libraryVersion

publishing {
    repositories {
        maven {
            name = "opentimestampsGitHubPackages"
            url = uri("https://maven.pkg.github.com/lvaccaro/rust-opentimestamps-clients")
            credentials {
                username = System.getenv("ACTOR_GITHUB")
                password = System.getenv("TOKEN_GITHUB")
            }
        }
    }

    publications {
        this.forEach {
            (it as MavenPublication).apply {
                pom {
                    name.set("ots-kmp")
                    description.set("Rust client for OpenTimestamps timestamps.")
                    url.set("https://opentimestamps.org")
                    licenses {
                        license {
                            name.set("LGPL3")
                            url.set("https://github.com/lvaccaro/rust-opentimestamps-client/blob/main/LICENSE")
                        }
                    }
                    scm {
                        connection.set("scm:git:github.com/lvaccaro/rust-opentimestamps-client.git")
                        developerConnection.set("scm:git:ssh://github.com/lvaccaro/rust-opentimestamps-client.git")
                        url.set("https://github.com/lvaccaro/rust-opentimestamps-client")
                    }
                }
            }
        }
    }
}