// The JVM wrapper around `ktrs serve` (io.github.hexay:ktrs). Tests run against the workspace's
// debug build: `cargo build --bins` first.
plugins {
    `java-library`
    id("com.vanniktech.maven.publish") version "0.37.0"
}

group = "io.github.hexay"
version = providers.gradleProperty("ktrsVersion").getOrElse("0.2.0")

// Two destinations (see .github/workflows/release.yml): the Maven repo served by GitHub Pages
// (`publishAllPublicationsToGithubPagesRepository -PpagesRepo=<checkout of gh-pages>/maven`), and
// Maven Central, which needs mavenCentralUsername/Password and signingInMemoryKey[Password].
publishing {
    repositories {
        maven {
            name = "githubPages"
            url = uri(providers.gradleProperty("pagesRepo").getOrElse(layout.buildDirectory.dir("pages-repo").get().asFile.path))
        }
    }
}

mavenPublishing {
    publishToMavenCentral()
    if (providers.gradleProperty("signingInMemoryKey").isPresent) {
        signAllPublications()
    }
    coordinates("io.github.hexay", "ktrs", version.toString())
    pom {
        name = "ktrs"
        description = "Formats Kotlin exactly like ktfmt 0.64, via a bundled native binary (no ktfmt on the classpath)."
        url = "https://github.com/Hexay/ktrs"
        licenses {
            license {
                name = "MIT"
                url = "https://github.com/Hexay/ktrs/blob/master/LICENSE-MIT"
            }
            license {
                name = "Apache-2.0"
                url = "https://github.com/Hexay/ktrs/blob/master/LICENSE-APACHE"
            }
        }
        developers {
            developer {
                id = "Hexay"
                name = "Hexay"
                url = "https://github.com/Hexay"
            }
        }
        scm {
            url = "https://github.com/Hexay/ktrs"
            connection = "scm:git:https://github.com/Hexay/ktrs.git"
        }
    }
}

tasks.withType<JavaCompile>().configureEach {
    options.release = 11
}

repositories {
    mavenCentral()
}

dependencies {
    // Only KtrsStep uses it; the Spotless plugin supplies it at runtime.
    compileOnly("com.diffplug.spotless:spotless-lib:3.0.0")
    testImplementation("com.diffplug.spotless:spotless-lib:3.0.0")
    testImplementation(platform("org.junit:junit-bom:5.13.4"))
    testImplementation("org.junit.jupiter:junit-jupiter")
    testRuntimeOnly("org.junit.platform:junit-platform-launcher")
}

// Bundled binaries: <nativeDir>/<platform>/ktrs[.exe], platforms as in NativeBinary.platform().
// Release CI fills it from the GitHub release archives; a local build without it bundles nothing.
val nativeDir = layout.projectDirectory.dir(providers.gradleProperty("nativeDir").getOrElse("native"))
tasks.jar {
    manifest { attributes("Implementation-Version" to version) }
}

tasks.processResources {
    from(nativeDir) { into("io/github/hexay/ktrs/native") }
}

tasks.test {
    useJUnitPlatform()
    val exe = if (System.getProperty("os.name").startsWith("Windows")) "ktrs.exe" else "ktrs"
    val executable = providers.gradleProperty("ktrsExecutable").getOrElse(rootDir.resolve("../target/debug/$exe").path)
    systemProperty("ktrs.executable", file(executable).absolutePath)
}
