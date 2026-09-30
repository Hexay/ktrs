import org.gradle.plugin.compatibility.compatibility
import org.jetbrains.kotlin.gradle.dsl.JvmTarget
import org.jetbrains.kotlin.gradle.dsl.KotlinVersion

// `io.github.hexay.ktrs`: a drop-in for cortinico's ktfmt-gradle 0.27.0 that formats through the
// root project's `ktrs serve` wrapper. Tests: `cargo build --bins`, then `java/gradlew -p java :ktrs-gradle-plugin:test`.
plugins {
    kotlin("jvm") version "2.4.10"
    `java-gradle-plugin`
    id("com.gradle.plugin-publish") version "2.1.1"
}

group = "io.github.hexay"
version = rootProject.version

java {
    sourceCompatibility = JavaVersion.VERSION_17
    targetCompatibility = JavaVersion.VERSION_17
}

kotlin {
    compilerOptions {
        apiVersion.set(KotlinVersion.KOTLIN_2_0)
        languageVersion.set(KotlinVersion.KOTLIN_2_0)
        jvmTarget = JvmTarget.JVM_17
    }
    explicitApi()
}

repositories {
    google()
    mavenCentral()
}

// TestKit builds see compileOnly (the Kotlin and Android Gradle plugins) through the injected plugin classpath, as upstream.
val integrationTestRuntime: Configuration by configurations.creating {
    extendsFrom(configurations.compileOnly.get())
    attributes {
        attribute(Attribute.of("org.gradle.usage", String::class.java), "java-runtime")
        attribute(Attribute.of("org.gradle.category", String::class.java), "library")
    }
}

tasks.withType<PluginUnderTestMetadata>().configureEach {
    pluginClasspath.from(integrationTestRuntime)
}

dependencies {
    implementation(project(":"))
    implementation("io.github.java-diff-utils:java-diff-utils:4.17")

    compileOnly(kotlin("gradle-plugin"))
    compileOnly("com.android.tools.build:gradle:9.3.1")

    testImplementation(kotlin("gradle-plugin"))
    testImplementation("com.android.tools.build:gradle:9.3.1")
    testImplementation(platform("org.junit:junit-bom:5.13.4"))
    testImplementation("org.junit.jupiter:junit-jupiter")
    testImplementation("com.google.truth:truth:1.4.5")
    testRuntimeOnly("org.junit.platform:junit-platform-launcher")
}

gradlePlugin {
    website = "https://github.com/Hexay/ktrs"
    vcsUrl = "https://github.com/Hexay/ktrs"
    plugins {
        create("ktrs") {
            id = "io.github.hexay.ktrs"
            implementationClass = "io.github.hexay.ktrs.gradle.KtrsPlugin"
            displayName = "ktrs (ktfmt-gradle drop-in)"
            description = "Drop-in replacement for com.ncorti.ktfmt.gradle: same DSL and tasks, formatting through " +
                "ktrs, a native port of ktfmt 0.64 with byte-identical output."
            tags = listOf("kotlin", "ktfmt", "formatter")
            compatibility { features { configurationCache = true } }
        }
    }
}

// Same Pages repo as the root jar (see ../build.gradle.kts); the Plugin Portal gets it via `publishPlugins`.
publishing {
    repositories {
        maven {
            name = "githubPages"
            url = uri(providers.gradleProperty("pagesRepo").getOrElse(layout.buildDirectory.dir("pages-repo").get().asFile.path))
        }
    }
}

val persistKtrsVersion by tasks.registering {
    val versionFile = layout.buildDirectory.file("ktrs-version/ktrs-version.txt")
    inputs.property("ktrsVersion", version.toString())
    outputs.file(versionFile)
    doLast { versionFile.get().asFile.writeText(inputs.properties["ktrsVersion"].toString()) }
}

tasks.processResources {
    from(persistKtrsVersion) { into("io/github/hexay/ktrs/gradle") }
}

tasks.test {
    useJUnitPlatform()
    systemProperty("ktrs.executable", rootProject.extra["testExecutable"] as String)
}
