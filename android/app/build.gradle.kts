import java.security.MessageDigest
import org.gradle.api.tasks.Sync

plugins {
    id("com.android.application")
}

val repositoryRoot = rootProject.projectDir.parentFile
val repositoryAssets = repositoryRoot.resolve("assets")
val generatedAssets = layout.buildDirectory.dir("generated/veloren-assets")
val generatedJniLibs = layout.buildDirectory.dir("generated/jniLibs")
val androidMinSdk = 26
val androidVersionCode = providers.gradleProperty("androidVersionCode").orElse("1").get().toInt()
val androidVersionName = providers.gradleProperty("androidVersionName").orElse("0.18.0").get()
val releaseStoreFile = providers.environmentVariable("ANDROID_RELEASE_KEYSTORE_PATH").orNull
val releaseStorePassword = providers.environmentVariable("ANDROID_RELEASE_KEYSTORE_PASSWORD").orNull
val releaseKeyAlias = providers.environmentVariable("ANDROID_RELEASE_KEY_ALIAS").orNull
val releaseKeyPassword = providers.environmentVariable("ANDROID_RELEASE_KEY_PASSWORD").orNull
val releaseSigningConfigured = listOf(
    releaseStoreFile,
    releaseStorePassword,
    releaseKeyAlias,
    releaseKeyPassword,
).all { !it.isNullOrBlank() }

android {
    namespace = "net.veloren.android"
    compileSdk = 35
    ndkVersion = "28.0.13004108"

    defaultConfig {
        applicationId = "net.veloren.android"
        minSdk = androidMinSdk
        targetSdk = 35
        versionCode = androidVersionCode
        versionName = androidVersionName

        ndk {
            abiFilters += "arm64-v8a"
        }
    }

    signingConfigs {
        if (releaseSigningConfigured) {
            create("release") {
                storeFile = file(requireNotNull(releaseStoreFile))
                storePassword = requireNotNull(releaseStorePassword)
                keyAlias = requireNotNull(releaseKeyAlias)
                keyPassword = requireNotNull(releaseKeyPassword)
            }
        }
    }

    buildTypes {
        release {
            isMinifyEnabled = false
            if (releaseSigningConfigured) {
                signingConfig = signingConfigs.getByName("release")
            }
        }
    }

    packaging {
        jniLibs.useLegacyPackaging = true
    }

    sourceSets.getByName("main").apply {
        assets.srcDir(generatedAssets)
        jniLibs.srcDir(generatedJniLibs)
    }
}

// Stage the repository's game data as ordinary APK assets. The generated index
// avoids relying on directory enumeration semantics of Android's AssetManager,
// and the digest lets the app skip the ~450 MB first-run extraction on later starts.
val stageVelorenAssets = tasks.register<Sync>("stageVelorenAssets") {
    from(repositoryAssets)
    into(generatedAssets.map { it.dir("veloren-assets") })

    doLast {
        val assetRoot = generatedAssets.get().dir("veloren-assets").asFile
        val canary = assetRoot.resolve("common/canary.canary")
        require(canary.isFile && canary.readText().startsWith("VELOREN_CANARY_MAGIC")) {
            "Veloren assets are missing or Git LFS files have not been fetched: ${canary.absolutePath}"
        }

        val files = assetRoot.walkTopDown()
            .filter { it.isFile && it.name != "veloren-assets-index.txt" && it.name != "veloren-assets-version.txt" }
            .sortedBy { it.relativeTo(assetRoot).invariantSeparatorsPath }
            .toList()
        val digest = MessageDigest.getInstance("SHA-256")
        files.forEach { file ->
            val relativePath = file.relativeTo(assetRoot).invariantSeparatorsPath
            digest.update(relativePath.toByteArray(Charsets.UTF_8))
            digest.update(0.toByte())
            file.inputStream().use { input ->
                val buffer = ByteArray(64 * 1024)
                while (true) {
                    val count = input.read(buffer)
                    if (count < 0) break
                    digest.update(buffer, 0, count)
                }
            }
        }
        val version = digest.digest().joinToString("") { "%02x".format(it.toInt() and 0xff) }
        assetRoot.resolve("veloren-assets-index.txt").writeText(
            files.joinToString(separator = "\n", postfix = "\n") {
                it.relativeTo(assetRoot).invariantSeparatorsPath
            },
        )
        assetRoot.resolve("veloren-assets-version.txt").writeText("$version\n")
    }
}

// cargo-ndk selects the Rust Android target, NDK clang/linker and native-library
// output layout. Cargo's own cache keeps this repeated Gradle task incremental.
val buildRustAndroid = tasks.register<Exec>("buildRustAndroid") {
    val outputDirectory = generatedJniLibs.get().asFile
    workingDir(repositoryRoot)
    outputs.dir(outputDirectory)
    outputs.upToDateWhen { false }

    doFirst {
        outputDirectory.mkdirs()
        val cmakeBin = android.sdkDirectory.resolve("cmake/3.22.1/bin")
        val inheritedPath = System.getenv("PATH").orEmpty()
        environment(
            "PATH",
            listOf(cmakeBin.absolutePath, inheritedPath)
                .filter { it.isNotBlank() }
                .joinToString(java.io.File.pathSeparator),
        )
        environment("ANDROID_NDK_HOME", android.ndkDirectory.absolutePath)
        commandLine(
            "cargo", "ndk",
            "-t", "arm64-v8a",
            "--platform", androidMinSdk.toString(),
            "-o", outputDirectory.absolutePath,
            "build",
            "--release",
            "--locked",
            "--package", "veloren-voxygen",
            "--lib",
            "--no-default-features",
            "--features", "android",
        )
    }
}

tasks.named("preBuild").configure {
    dependsOn(stageVelorenAssets, buildRustAndroid)
}

val verifyReleaseSigning = tasks.register("verifyReleaseSigning") {
    doLast {
        check(releaseSigningConfigured) {
            "Release APKs must be signed; configure the ANDROID_RELEASE_KEYSTORE_* environment variables."
        }
        check(file(requireNotNull(releaseStoreFile)).isFile) {
            "ANDROID_RELEASE_KEYSTORE_PATH does not point to an existing keystore."
        }
    }
}

tasks.named("preReleaseBuild").configure {
    dependsOn(verifyReleaseSigning)
}
