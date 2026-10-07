// The JVM wrapper around `ktrs serve` (io.github.hexay:ktrs). Tests run against the workspace's
// debug build: `cargo build --bins` first.
plugins {
    `java-library`
    id("com.vanniktech.maven.publish") version "0.37.0"
}

group = "io.github.hexay"
version = providers.gradleProperty("ktrsVersion").getOrElse("0.4.0")

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
        description = "Formats and lints Kotlin exactly like ktfmt 0.64 and ktlint 1.8/2.0, via a bundled native binary " +
            "(no ktfmt or ktlint on the classpath)."
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

// The tests run against the latest Spotless, which needs Java 17.
tasks.compileTestJava {
    options.release = 17
}

repositories {
    mavenCentral()
}

// KtrsKtfmt/KtrsKtlint: spotless-maven-plugin's `implementation=` swap (research/30). Their own source set because
// they compile against spotless-maven-plugin's spotless-lib 4.x and Java 17, while main stays on lib 3.0.0 and Java 11.
val spotlessMaven: SourceSet by sourceSets.creating {
    compileClasspath += sourceSets.main.get().output
}
tasks.named<JavaCompile>("compileSpotlessMavenJava") {
    options.release = 17
}
tasks.jar { from(spotlessMaven.output) }
tasks.withType<Jar>().matching { it.name == "sourcesJar" }.configureEach { from(spotlessMaven.allSource) }
sourceSets.test {
    compileClasspath += spotlessMaven.output
    runtimeClasspath += spotlessMaven.output
}
val spotlessMavenPlugin = "com.diffplug.spotless:spotless-maven-plugin:3.10.3"

// Spotless's own ktlint() step runs ktlint from these in SpotlessKtlintParityTest.
val spotlessKtlint: Configuration by configurations.creating
val spotlessKtlintCompose: Configuration by configurations.creating

dependencies {
    // Only the spotless steps use it (compiled against the oldest supported release); Spotless supplies it at runtime.
    compileOnly("com.diffplug.spotless:spotless-lib:3.0.0")
    "spotlessMavenCompileOnly"(spotlessMavenPlugin)
    // The plugin's POM has it at runtime scope only.
    "spotlessMavenCompileOnly"("com.diffplug.spotless:spotless-lib:4.10.3")
    testImplementation("com.diffplug.spotless:spotless-lib:4.10.3")
    testImplementation(spotlessMavenPlugin)
    // spotless-lib logs through it; the Spotless plugins bring it along.
    testRuntimeOnly("org.slf4j:slf4j-simple:2.0.17")
    spotlessKtlint("com.pinterest.ktlint:ktlint-cli:1.8.0")
    spotlessKtlintCompose("com.pinterest.ktlint:ktlint-cli:1.8.0")
    spotlessKtlintCompose("io.nlopez.compose.rules:ktlint:0.6.7")
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

// The binary tests run against (this project's and gradle-plugin's): -PktrsExecutable, else the workspace's debug build.
val testExecutable: String by extra(file(providers.gradleProperty("ktrsExecutable").getOrElse(
    rootDir.resolve("../target/debug/" + if (System.getProperty("os.name").startsWith("Windows")) "ktrs.exe" else "ktrs").path
)).absolutePath)

tasks.test {
    useJUnitPlatform()
    systemProperty("ktrs.executable", testExecutable)
    inputs.files(spotlessKtlint, spotlessKtlintCompose)
    // Optional: a directory of Kotlin files for the ktlint parity tests (e.g. the repo's corpus/), and a file cap.
    providers.gradleProperty("ktlintParityCorpus").orNull?.let { systemProperty("ktrs.ktlintParityCorpus", it) }
    providers.gradleProperty("ktlintParityMaxFiles").orNull?.let { systemProperty("ktrs.ktlintParityMaxFiles", it) }
    doFirst {
        systemProperty("spotless.ktlint.classpath", spotlessKtlint.asPath)
        systemProperty("spotless.ktlintCompose.classpath", spotlessKtlintCompose.asPath)
    }
}
