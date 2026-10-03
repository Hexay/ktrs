package com.ncorti.ktfmt.gradle.testutil

import java.io.File
import org.gradle.testkit.runner.GradleRunner
import org.intellij.lang.annotations.Language

/** Copies a fixture from src/test/resources and points the plugin at the ktrs binary under test. */
fun File.copyFixture(name: String) {
    File("src/test/resources/$name").copyRecursively(this)
    val executable = System.getProperty("ktrs.executable").replace('\\', '/')
    resolve("gradle.properties").writeText("ktrs.executable=$executable\n")
}

fun File.gradle(vararg arguments: String): GradleRunner =
    GradleRunner.create().withProjectDir(this).withPluginClasspath().withArguments(*arguments)

fun File.appendToBuildGradle(content: String) {
    resolve("build.gradle.kts").apply {
        // LF like the fixture: CRLF lines appended on Windows would make `ktfmtCheckScripts` flag the build script.
        appendText("\n")
        appendText(content.replace("\r\n", "\n"))
        appendText("\n")
    }
}

fun File.createTempFile(
    @Language("kotlin") content: String,
    fileName: String = "TestFile.kt",
    path: String = "src/main/java",
): File =
    resolve(path).resolve(fileName).apply {
        parentFile.mkdirs()
        createNewFile()
        writeText(content)
    }
