// `io.github.hexay:ktrs-ktlint-maven-plugin`: a drop-in for gantsign's ktlint-maven-plugin 3.7.1 over `ktrs ktlint`
// (research/31-ktlint-maven-dropin.md). End to end vs the real plugin: tools/ktlint-maven/parity.sh.
plugins {
    `java-library`
    id("org.gradlex.maven-plugin-development") version "1.0.3"
    id("com.vanniktech.maven.publish")
}

group = "io.github.hexay"
version = rootProject.version

repositories {
    mavenCentral()
}

tasks.withType<JavaCompile>().configureEach {
    options.release = 11
}

val mavenVersion = "3.9.0"

dependencies {
    implementation(project(":"))
    implementation("org.apache.maven.shared:maven-shared-utils:3.4.2")
    // gantsign's report base (Doxia 1, maven-site-plugin 3.x) and its exclusions.
    implementation("org.apache.maven.reporting:maven-reporting-impl:3.1.0") {
        exclude(group = "com.google.collections", module = "google-collections")
        exclude(group = "org.codehaus.plexus", module = "plexus-container-default")
        exclude(group = "org.apache.struts")
        exclude(group = "dom4j", module = "dom4j")
    }
    runtimeOnly("org.dom4j:dom4j:2.1.5")

    compileOnly("org.apache.maven:maven-plugin-api:$mavenVersion")
    compileOnly("org.apache.maven:maven-core:$mavenVersion")
    compileOnly("org.apache.maven.plugin-tools:maven-plugin-annotations:3.15.1")

    testImplementation("org.apache.maven:maven-plugin-api:$mavenVersion")
    testImplementation("org.apache.maven:maven-core:$mavenVersion")
    testImplementation(platform("org.junit:junit-bom:5.13.4"))
    testImplementation("org.junit.jupiter:junit-jupiter")
    testRuntimeOnly("org.junit.platform:junit-platform-launcher")
}

mavenPlugin {
    goalPrefix = "ktlint"
    helpMojoPackage = "io.github.hexay.ktrs.maven.ktlint"
}

// What the plugin itself brings: `JarServices` keeps only the jars users add as plugin <dependencies>.
val persistPluginArtifacts by tasks.registering {
    val listFile = layout.buildDirectory.file("plugin-artifacts/plugin-artifacts.txt")
    val ids = configurations.runtimeClasspath.flatMap { classpath ->
        classpath.incoming.resolutionResult.rootComponent.map { root -> reachableModules(root) }
    }
    inputs.property("ids", ids)
    outputs.file(listFile)
    doLast { listFile.get().asFile.writeText(ids.get().joinToString("\n", postfix = "\n")) }
}

/** `group:name` of every component [root] depends on, transitively (the project dependency `:` included). */
fun reachableModules(root: org.gradle.api.artifacts.result.ResolvedComponentResult): List<String> {
    val seen = mutableSetOf(root)
    val queue = ArrayDeque(listOf(root))
    while (queue.isNotEmpty()) {
        queue.removeFirst().dependencies
            .mapNotNull { (it as? org.gradle.api.artifacts.result.ResolvedDependencyResult)?.selected }
            .filter(seen::add)
            .forEach(queue::add)
    }
    return (seen - root).mapNotNull { it.moduleVersion }.map { "${it.group}:${it.name}" }.sorted()
}

tasks.processResources {
    from(persistPluginArtifacts) { into("io/github/hexay/ktrs/maven/ktlint") }
}

// Same Pages repo as the root jar (see ../build.gradle.kts), which must be published first.
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
    coordinates("io.github.hexay", "ktrs-ktlint-maven-plugin", version.toString())
    pom {
        name = "ktrs ktlint Maven plugin"
        packaging = "maven-plugin"
        description = "Drop-in replacement for com.github.gantsign.maven:ktlint-maven-plugin 3.7.1: same goals, " +
            "parameters and reports, linting through ktrs, a native port of ktlint 1.8.0 and 2.0.0-ALPHA-4."
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

// The generated HelpMojo carries legacy `@goal`-style tags, which doclint rejects.
tasks.javadoc {
    (options as StandardJavadocDocletOptions).addStringOption("Xdoclint:none", "-quiet")
}

tasks.test {
    useJUnitPlatform()
    systemProperty("ktrs.executable", rootProject.extra["testExecutable"] as String)
}
