// The ktrs plugin for IntelliJ IDEA and Android Studio: a client of `ktrs lsp` (see README.md).
import org.jetbrains.intellij.platform.gradle.IntelliJPlatformType
import org.jetbrains.intellij.platform.gradle.TestFrameworkType

plugins {
    id("org.jetbrains.kotlin.jvm") version "2.4.20"
    id("org.jetbrains.intellij.platform") version "2.19.0"
}

group = "io.github.hexay"
version = providers.gradleProperty("ktrsVersion").getOrElse("0.0.0-dev")

val ideaVersion = "2025.3.6.1"
val androidStudioVersion = "2026.2.1.8"

kotlin {
    jvmToolchain(21)
}

dependencies {
    intellijPlatform {
        intellijIdea(ideaVersion)
        plugin("com.redhat.devtools.lsp4ij:0.21.0")
        testFramework(TestFrameworkType.Platform)
        pluginVerifier()
    }
    testImplementation("junit:junit:4.13.2")
    testImplementation("org.opentest4j:opentest4j:1.3.0")
}

intellijPlatform {
    buildSearchableOptions = false
    instrumentCode = false
    pluginConfiguration {
        ideaVersion {
            sinceBuild = "253"
            untilBuild = provider { null }
        }
    }
    pluginVerification {
        ides {
            create(IntelliJPlatformType.IntellijIdea, ideaVersion)
            if (providers.gradleProperty("verifyAndroidStudio").isPresent) {
                create(IntelliJPlatformType.AndroidStudio, androidStudioVersion)
            }
        }
    }
    // The first version of a new plugin must be uploaded by hand on plugins.jetbrains.com; this publishes the rest.
    publishing {
        token = providers.environmentVariable("JETBRAINS_MARKETPLACE_TOKEN")
    }
}

// Bundled binaries: <nativeDir>/<platform>/ktrs[.exe] (tools/release/bundle-natives.sh), shipped as <plugin>/bin/.
// A build without it bundles nothing and runs `ktrs` from PATH or the configured path.
val nativeDir = layout.projectDirectory.dir(providers.gradleProperty("nativeDir").getOrElse("bin"))
tasks.prepareSandbox {
    from(nativeDir) {
        into(intellijPlatform.projectName.map { "$it/bin" })
        filePermissions { unix("rwxr-xr-x") }
    }
}

// The binary the integration test runs: -PktrsExecutable, else the workspace's debug build.
val testExecutable = file(providers.gradleProperty("ktrsExecutable").getOrElse(
    "../../target/debug/" + if (System.getProperty("os.name").startsWith("Windows")) "ktrs.exe" else "ktrs"
)).absolutePath

tasks.test {
    systemProperty("ktrs.testExecutable", testExecutable)
}
