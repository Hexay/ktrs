package com.ncorti.ktfmt.gradle.tasks

import com.google.common.truth.Truth.assertThat
import com.ncorti.ktfmt.gradle.testutil.appendToBuildGradle
import com.ncorti.ktfmt.gradle.testutil.copyFixture
import com.ncorti.ktfmt.gradle.testutil.createTempFile
import com.ncorti.ktfmt.gradle.testutil.gradle
import java.io.File
import org.gradle.testkit.runner.BuildResult
import org.gradle.testkit.runner.TaskOutcome.FAILED
import org.gradle.testkit.runner.TaskOutcome.FROM_CACHE
import org.gradle.testkit.runner.TaskOutcome.NO_SOURCE
import org.gradle.testkit.runner.TaskOutcome.SUCCESS
import org.gradle.testkit.runner.TaskOutcome.UP_TO_DATE
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import org.junit.jupiter.params.ParameterizedTest
import org.junit.jupiter.params.provider.ValueSource

internal class KtfmtCheckTaskIntegrationTest {

    @TempDir lateinit var tempDir: File

    @BeforeEach
    fun setUp() {
        tempDir.copyFixture("jvmProject")
    }

    @Test
    fun `check task fails if there is invalid code`() {
        tempDir.createTempFile(content = "val answer = `")
        val result = tempDir.gradle("ktfmtCheckMain").buildAndFail()

        assertThat(result.task(":ktfmtCheckMain")?.outcome).isEqualTo(FAILED)
        assertThat(result.output).contains("Ktfmt failed to run with 1 failures")
    }

    @Test
    fun `check task fails if there is not formatted code`() {
        tempDir.createTempFile(content = "val answer=42")
        val result = tempDir.gradle("ktfmtCheckMain").buildAndFail()

        assertThat(result.task(":ktfmtCheckMain")?.outcome).isEqualTo(FAILED)
        assertThat(result.output).contains("[ktfmt] Invalid formatting")
    }

    @Test
    fun `check task fails if ktfmt fails to parse the code`() {
        tempDir.createTempFile(
            """
            val res = when {
                ````
            }
            """
                .trimIndent()
        )

        val result = tempDir.gradle("ktfmtCheckMain", "--info").buildAndFail()

        assertThat(result.task(":ktfmtCheckMain")?.outcome).isEqualTo(FAILED)
        assertThat(result.output).containsMatch("Failed to format file: .*TestFile.kt \\(reason =")
        assertThat(result.output).contains("Ktfmt failed to run with 1 failures")
        assertThat(result.output)
            .contains("src${File.separator}main${File.separator}java${File.separator}TestFile.kt")
    }

    @Test
    fun `check task succeed if code is formatted`() {
        tempDir.createTempFile(content = "val answer = 42\n")
        val result = tempDir.gradle("ktfmtCheckMain").build()

        assertThat(result.task(":ktfmtCheckMain")?.outcome).isEqualTo(SUCCESS)
    }

    @Test
    fun `check task runs before compilation`() {
        tempDir.createTempFile(content = "val answer = 42\n")
        val result = tempDir.gradle("compileKotlin", "ktfmtCheckMain", "--dry-run").build()

        assertThat(result.output).contains(":ktfmtCheckMain SKIPPED")
        assertThat(result.output).contains(":compileKotlin SKIPPED")
        assertThat(result.output.indexOf(":ktfmtCheckMain SKIPPED"))
            .isLessThan(result.output.indexOf(":compileKotlin SKIPPED"))
    }

    @Test
    fun `check task prints formatted files with --info`() {
        tempDir.createTempFile(content = "val answer = 42\n")
        val result = tempDir.gradle("ktfmtCheckMain", "--info").build()

        assertThat(result.task(":ktfmtCheckMain")?.outcome).isEqualTo(SUCCESS)
        assertThat(result.output).contains("[ktfmt] Successfully checked 1 files with Ktfmt")
        assertThat(result.output).contains("[ktfmt] Valid formatting")
    }

    @Test
    fun `check task prints the diff of not formatted code with --info`() {
        tempDir.createTempFile(content = "val answer = 42\nval  other = 1\n")
        val result = tempDir.gradle("ktfmtCheckMain", "--info").buildAndFail()

        assertThat(result.output).containsMatch("TestFile.kt:2 - Line changed: val  other = 1")
        assertThat(result.output)
            .contains("[ktfmt] Found 1 files that are not properly formatted:")
    }

    @Test
    fun `format task uses configuration cache correctly`() {
        tempDir.createTempFile(content = "val answer = 42\n")
        tempDir.gradle("--configuration-cache", "--rerun-tasks", "ktfmtCheckMain").build()

        val result =
            tempDir.gradle("--configuration-cache", "--rerun-tasks", "ktfmtCheckMain").build()

        assertThat(result.output).contains("Reusing configuration cache.")
    }

    @Test
    fun `check task validates all the file with a failure`() {
        tempDir.createTempFile(content = "val answer = `\n", fileName = "File1.kt")
        tempDir.createTempFile(content = "val answer = 42\n", fileName = "File2.kt")

        val result = tempDir.gradle("ktfmtCheckMain", "--info").buildAndFail()

        assertThat(result.task(":ktfmtCheckMain")?.outcome).isEqualTo(FAILED)
        assertThat(result.output).containsMatch("Failed to format file: .*File1.kt \\(reason =")
        assertThat(result.output).containsMatch("Valid formatting for: .*File2.kt")
    }

    @Test
    fun `check task skips a file if with --include-only`() {
        tempDir.createTempFile(content = "val answer = `\n", fileName = "File1.kt")
        val file2 = tempDir.createTempFile(content = "val answer = 42\n", fileName = "File2.kt")

        val result =
            tempDir
                .gradle("ktfmtCheckMain", "--debug", "--include-only=${file2.relativeTo(tempDir)}")
                .build()

        assertThat(result.task(":ktfmtCheckMain")?.outcome).isEqualTo(SUCCESS)
        assertThat(result.output).containsMatch("Valid formatting for: .*File2.kt")
        assertThat(result.output)
            .containsMatch("Skipping format for .*File1.kt because it is not included")
    }

    @ParameterizedTest
    @ValueSource(ints = [10, 15, 30, 50, 100, 1000])
    fun `check task can check the formatting of multiple files`(n: Int) {
        repeat(n) { index ->
            tempDir.createTempFile(
                content = "val answer${index} = 42\n",
                fileName = "TestFile$index.kt",
            )
        }
        val result = tempDir.gradle("ktfmtCheckMain", "--info").build()

        assertThat(result.task(":ktfmtCheckMain")?.outcome).isEqualTo(SUCCESS)
        assertThat(result.output).contains("[ktfmt] Successfully checked $n files with Ktfmt")
    }

    @Test
    fun `check task is cacheable`() {
        tempDir.createTempFile(content = "val answer = 42\n")

        var result: BuildResult? = null
        repeat(2) { result = tempDir.gradle("clean", "ktfmtCheckMain", "--build-cache").build() }

        assertThat(result!!.task(":ktfmtCheckMain")?.outcome).isEqualTo(FROM_CACHE)
    }

    @Test
    fun `check task should be up-to-date when invoked twice with multiple different sized sourceSets`() {
        tempDir.createTempFile(content = "val answer = 42\n", fileName = "SrcFile.kt")
        tempDir.createTempFile(content = "val answer = 42\n", path = "src/test/java")
        tempDir.createTempFile(
            content = "val answer = 42\n",
            fileName = "TestFile2.kt",
            path = "src/test/java",
        )

        val firstRun = tempDir.gradle("ktfmtCheck", "--info").build()

        assertThat(firstRun.task(":ktfmtCheckMain")?.outcome).isEqualTo(SUCCESS)
        assertThat(firstRun.task(":ktfmtCheckTest")?.outcome).isEqualTo(SUCCESS)

        val secondRun = tempDir.gradle("ktfmtCheck", "--info").build()

        assertThat(secondRun.task(":ktfmtCheckMain")?.outcome).isEqualTo(UP_TO_DATE)
        assertThat(secondRun.task(":ktfmtCheckTest")?.outcome).isEqualTo(UP_TO_DATE)
    }

    @Test
    fun `check task is configuration cache compatible`() {
        tempDir.createTempFile(content = "val answer = 42\n")

        var result: BuildResult? = null
        repeat(2) { result = tempDir.gradle("ktfmtCheckMain", "--configuration-cache").build() }

        assertThat(result!!.output).contains("Reusing configuration cache.")
    }

    @Test
    fun `custom formatCheck task should be compatible with configuration cache`() {
        tempDir.createTempFile(content = "val answer = 42\n")

        tempDir.appendToBuildGradle(
            """
            |tasks.register<com.ncorti.ktfmt.gradle.tasks.KtfmtCheckTask>("customFormatCheck") {
            |    source = fileTree("src/main/java")
            |}
            """
                .trimMargin()
        )

        tempDir.gradle("customFormatCheck", "--configuration-cache").build()
    }

    @Test
    fun `check task should detect the source and test files in a flattened project structure`() {
        tempDir.appendToBuildGradle(
            """
            |kotlin {
            |    sourceSets.main { kotlin.setSrcDirs(listOf("src")) }
            |    sourceSets.test { kotlin.setSrcDirs(listOf("test")) }
            |}
            """
                .trimMargin()
        )

        tempDir.createTempFile("val answer = 42\n", path = "src/someFolder")
        tempDir.createTempFile("val answer = 42\n", path = "test/someOtherFolder")

        val result = tempDir.gradle("ktfmtCheck").build()

        assertThat(result.task(":ktfmtCheckMain")?.outcome).isNotEqualTo(NO_SOURCE)
        assertThat(result.task(":ktfmtCheckTest")?.outcome).isNotEqualTo(NO_SOURCE)
    }

    @Test
    fun `check task should by default ignore sourceSets in the build folder`() {
        tempDir.appendToBuildGradle("kotlin { sourceSets.main { kotlin.srcDirs(\"build/main\") } }")
        tempDir.createTempFile(content = "val answer=42\n", path = "build/main")

        val result = tempDir.gradle("ktfmtCheck").build()

        assertThat(result.task(":ktfmtCheckMain")?.outcome).isEqualTo(NO_SOURCE)
    }

    @Test
    fun `check task should not ignore sourceSets in build folder when a custom exclusion pattern is specified`() {
        tempDir.appendToBuildGradle(
            """
            |kotlin { sourceSets.main { kotlin.srcDirs("build/generated") } }
            |
            |ktfmt { srcSetPathExclusionPattern.set(Regex("customRules.*")) }
            """
                .trimMargin()
        )

        tempDir.createTempFile(content = "val answer=42\n", path = "build/generated/main")

        val result = tempDir.gradle("ktfmtCheck").buildAndFail()

        assertThat(result.task(":ktfmtCheckMain")?.outcome).isEqualTo(FAILED)
        assertThat(result.output).containsMatch("Invalid formatting for: .*TestFile.kt")
    }

    @Test
    fun `check task should ignore the main sourceSets when specified as exclusion pattern`() {
        tempDir.appendToBuildGradle(
            "ktfmt { srcSetPathExclusionPattern.set(Regex(\".*[\\\\\\\\/]main[\\\\\\\\/].*\")) }"
        )
        tempDir.createTempFile(content = "val answer=42\n")

        val result = tempDir.gradle("ktfmtCheck").build()

        assertThat(result.task(":ktfmtCheckMain")?.outcome).isEqualTo(NO_SOURCE)
    }

    @Test
    fun `check scripts task should validate top-level script file`() {
        tempDir.createTempFile(content = "val answer=42\n", fileName = "TestFile.kts", path = "")

        val result = tempDir.gradle("ktfmtCheckScripts").buildAndFail()

        assertThat(result.task(":ktfmtCheckScripts")?.outcome).isEqualTo(FAILED)
        assertThat(result.output).containsMatch("Invalid formatting for: .*TestFile.kts")
    }

    @Test
    fun `check scripts task should ignore non top-level script files`() {
        tempDir.createTempFile(content = "val answer=42\n", fileName = "TestFile.kts")

        val result = tempDir.gradle("ktfmtCheckScripts").build()

        assertThat(result.task(":ktfmtCheckScripts")?.outcome).isEqualTo(SUCCESS)
    }

    @Test
    fun `format scripts task should validate top-level script file on project without any kotlin plugins`() {
        tempDir
            .resolve("build.gradle.kts")
            .writeText(
                """
                plugins {
                    id("io.github.hexay.ktrs")
                }

                repositories {
                    mavenCentral()
                }
                """
                    .trimIndent()
            )
        tempDir.resolve("script.kts").writeText("val x=1")

        val result = tempDir.gradle("ktfmtCheckScripts").buildAndFail()

        assertThat(result.output).contains("[ktfmt] Invalid formatting")
    }
}
