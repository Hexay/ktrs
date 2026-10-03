package org.jlleitschuh.gradle.ktlint.testdsl

import java.io.File
import org.gradle.testkit.runner.BuildResult
import org.gradle.testkit.runner.GradleRunner
import org.jlleitschuh.gradle.ktlint.AbstractPluginTest

const val PLUGIN_ID = "io.github.hexay.ktrs.ktlint"

fun AbstractPluginTest.project(
    projectPath: File = projectRoot,
    projectSetup: (File) -> Unit = defaultProjectSetup(),
    test: TestProject.() -> Unit = {},
): TestProject {
    projectSetup(projectPath)
    val gradleRunner = GradleRunner.create().withPluginClasspath().withProjectDir(projectPath)
    return TestProject(gradleRunner, projectPath).apply(test)
}

class TestProject(val gradleRunner: GradleRunner, val projectPath: File) {
    val buildGradle: File
        get() = projectPath.resolve("build.gradle")

    val settingsGradle: File
        get() = projectPath.resolve("settings.gradle")

    val editorConfig: File
        get() = projectPath.resolve(".editorconfig")

    fun withCleanSources(filePath: String = CLEAN_SOURCES_FILE) {
        createSourceFile(filePath, "val foo = \"bar\"\n")
    }

    fun withFailingSources() {
        createSourceFile(FAIL_SOURCE_FILE, "val  foo    =     \"bar\"\n")
    }

    fun withFailingMaxLineSources() {
        createSourceFile(
            FAIL_SOURCE_FILE,
            "val nameOfVariable =\n    listOf(1, 2, 3, 1, 2, 3, 1, 2, 3, 1, 2, 3, 1, 2, 3, 1, 2, 3, 1, 2, 3, 2)\n",
        )
    }

    fun withCleanKotlinScript() {
        createSourceFile("kotlin-script.kts", "println(\"zzz\")\n")
    }

    fun withFailingKotlinScript() {
        createSourceFile("kotlin-script-fail.kts", "println(\"zzz\") \n")
    }

    fun createSourceFile(sourceFilePath: String, contents: String) {
        val sourceFile = projectPath.resolve(sourceFilePath)
        sourceFile.parentFile.mkdirs()
        sourceFile.writeText(contents)
    }

    fun restoreFailingSources() {
        projectPath.resolve(FAIL_SOURCE_FILE).delete()
        withFailingSources()
    }

    fun removeSourceFile(sourceFilePath: String) {
        projectPath.resolve(sourceFilePath).delete()
    }

    companion object {
        const val CLEAN_SOURCES_FILE = "src/main/kotlin/CleanSource.kt"
        const val FAIL_SOURCE_FILE = "src/main/kotlin/FailSource.kt"
    }
}

fun TestProject.build(vararg buildArguments: String, assertions: BuildResult.() -> Unit = {}) {
    gradleRunner.withArguments(buildArguments.toList() + "--stacktrace").build().run(assertions)
}

fun TestProject.buildAndFail(vararg buildArguments: String, assertions: BuildResult.() -> Unit = {}) {
    gradleRunner.withArguments(buildArguments.toList() + "--stacktrace").buildAndFail().run(assertions)
}

fun defaultProjectSetup(): (File) -> Unit = projectSetup("jvm")

/** Upstream's setup, with the Kotlin plugin from TestKit's injected classpath and the ktrs binary under test. */
fun projectSetup(kotlinPluginType: String): (File) -> Unit = {
    it.mkdirs()
    it.resolve("build.gradle").writeText(
        """
        |plugins {
        |    id 'org.jetbrains.kotlin.$kotlinPluginType'
        |    id '$PLUGIN_ID'
        |}
        |
        |repositories {
        |    mavenCentral()
        |}
        |
        """
            .trimMargin()
    )
    it.resolve("settings.gradle").writeText("")
    it.writeKtrsExecutable()
}

fun File.writeKtrsExecutable() {
    val executable = System.getProperty("ktrs.executable").replace('\\', '/')
    resolve("gradle.properties").writeText("ktrs.executable=$executable\n")
}
