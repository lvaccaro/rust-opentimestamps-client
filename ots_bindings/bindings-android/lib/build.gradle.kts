plugins {
    id("com.android.library")
    id("org.jetbrains.kotlin.android") version "1.8.20"
    id("maven-publish")
    kotlin("plugin.serialization") version "1.8.20"
}

repositories {
    mavenCentral()
    google()
}

android {
    compileSdk = 33

    defaultConfig {
        minSdk = 24
        consumerProguardFiles("consumer-rules.pro")
    }

    kotlinOptions {
        jvmTarget = "1.8"
    }

    buildTypes {
        getByName("release") {
            @Suppress("UnstableApiUsage")
            isMinifyEnabled = false
            proguardFiles(file("proguard-android-optimize.txt"), file("proguard-rules.pro"))
        }
    }

    publishing {
        singleVariant("release") {
            withSourcesJar()
        }
    }
}

dependencies {
    implementation("net.java.dev.jna:jna:5.14.0@aar")
    implementation("org.jetbrains.kotlin:kotlin-stdlib-jdk7")
    implementation("androidx.appcompat:appcompat:1.6.1")
    implementation("org.jetbrains.kotlinx:kotlinx-serialization-json:1.6.3")
}

val libraryVersion: String by project

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
        create<MavenPublication>("maven") {
            groupId = "ots"
            artifactId = "bindings-android"
            version = libraryVersion

            afterEvaluate {
                from(components["release"])
            }

            pom {
                name.set("Opentimestamps")
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
