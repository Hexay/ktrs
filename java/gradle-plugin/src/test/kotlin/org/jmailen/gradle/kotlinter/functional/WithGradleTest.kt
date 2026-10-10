package org.jmailen.gradle.kotlinter.functional

import java.io.File
import java.nio.file.Files
import org.gradle.internal.classpath.DefaultClassPath
import org.gradle.testkit.runner.BuildResult
import org.gradle.testkit.runner.GradleRunner
import org.gradle.testkit.runner.internal.PluginUnderTestMetadataReading
import org.jmailen.gradle.kotlinter.functional.utils.ktrsExecutableArgument
import org.junit.jupiter.api.AfterEach
import org.junit.jupiter.api.BeforeEach

abstract class WithGradleTest {

    lateinit var testProjectDir: File

    // Not JUnit's @TempDir: https://github.com/gradle/gradle/issues/12535
    @BeforeEach
    internal fun setUpTempdir() {
        testProjectDir = Files.createTempDirectory(this::class.java.simpleName).toFile()
    }

    @AfterEach
    internal fun cleanUpTempdir() {
        testProjectDir.deleteRecursively()
    }

    protected fun build(vararg args: String): BuildResult = gradleRunnerFor(*args).build()

    protected fun buildAndFail(vararg args: String): BuildResult = gradleRunnerFor(*args).buildAndFail()

    protected abstract fun gradleRunnerFor(vararg args: String): GradleRunner

    abstract class Android : WithGradleTest() {

        override fun gradleRunnerFor(vararg args: String): GradleRunner = defaultRunner(*args).withPluginClasspath()
    }

    abstract class Kotlin : WithGradleTest() {

        override fun gradleRunnerFor(vararg args: String): GradleRunner {
            val classpath = DefaultClassPath.of(PluginUnderTestMetadataReading.readImplementationClasspath()).asFiles
            val androidDependencies =
                listOf(
                    ".*/com\\.android\\..*/.*".toRegex(),
                    ".*/androidx\\..*/.*".toRegex(),
                    ".*/com\\.google\\..*/.*".toRegex(),
                )
            val noAndroid =
                classpath.filterNot { dependency ->
                    androidDependencies.any { it.matches(dependency.invariantSeparatorsPath) }
                }

            return defaultRunner(*args).withPluginClasspath(noAndroid)
        }
    }
}

private fun WithGradleTest.defaultRunner(vararg args: String) =
    GradleRunner.create()
        .withProjectDir(testProjectDir)
        .withArguments(args.toList() + listOf(ktrsExecutableArgument, "--stacktrace", "--configuration-cache"))
        .forwardOutput()
