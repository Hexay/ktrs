package com.ncorti.ktfmt.gradle.tasks

import com.google.common.truth.Truth.assertThat
import com.ncorti.ktfmt.gradle.testutil.appendToBuildGradle
import com.ncorti.ktfmt.gradle.testutil.copyFixture
import com.ncorti.ktfmt.gradle.testutil.createTempFile
import com.ncorti.ktfmt.gradle.testutil.gradle
import java.io.File
import org.gradle.testkit.runner.TaskOutcome.FAILED
import org.gradle.testkit.runner.TaskOutcome.SUCCESS
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import org.junit.jupiter.params.ParameterizedTest
import org.junit.jupiter.params.provider.ValueSource

/** Styles, source sets and scripts: [KtfmtFormatTaskSourcesIntegrationTest]. */
internal class KtfmtFormatTaskIntegrationTest {

    @TempDir lateinit var tempDir: File

    @BeforeEach
    fun setUp() {
        tempDir.copyFixture("jvmProject")
    }

    @Test
    fun `format task fails if there is invalid code`() {
        tempDir.createTempFile(content = "val answer = `")
        val result = tempDir.gradle("ktfmtFormatMain").buildAndFail()

        assertThat(result.task(":ktfmtFormatMain")?.outcome).isEqualTo(FAILED)
        assertThat(result.output).containsMatch("Failed to format file: .*TestFile.kt \\(reason =")
        assertThat(result.output).contains("Ktfmt failed to run with 1 failures")
        assertThat(result.output)
            .contains("src${File.separator}main${File.separator}java${File.separator}TestFile.kt")
    }

    @Test
    fun `format formats correctly`() {
        val tempFile = tempDir.createTempFile(content = "val answer=42")
        val result = tempDir.gradle("ktfmtFormatMain").build()

        assertThat(result.task(":ktfmtFormatMain")?.outcome).isEqualTo(SUCCESS)
        assertThat(tempFile.readText()).isEqualTo("val answer = 42\n")
    }

    @Test
    fun `format task succeed if code is formatted`() {
        tempDir.createTempFile(content = "val answer = 42\n")
        val result = tempDir.gradle("ktfmtFormatMain").build()

        assertThat(result.task(":ktfmtFormatMain")?.outcome).isEqualTo(SUCCESS)
    }

    @Test
    fun `format task succeeds after subsequent execution`() {
        tempDir.createTempFile(content = "val answer = 42\n")
        repeat(2) {
            val result = tempDir.gradle("ktfmtFormatMain").build()
            // SUCCESS, never UP_TO_DATE: the task has no outputs, as it edits its inputs.
            assertThat(result.task(":ktfmtFormatMain")?.outcome).isEqualTo(SUCCESS)
        }
    }

    @Test
    fun `format task succeeds after subsequent execution when formatting`() {
        tempDir.createTempFile(content = "val answer=42")
        repeat(3) {
            val result = tempDir.gradle("ktfmtFormatMain").build()
            assertThat(result.task(":ktfmtFormatMain")?.outcome).isEqualTo(SUCCESS)
        }
    }

    @Test
    fun `format task is executed again after edit`() {
        val tempFile = tempDir.createTempFile(content = "val answer = 42\n")
        repeat(2) {
            val result = tempDir.gradle("ktfmtFormatMain").build()
            assertThat(result.task(":ktfmtFormatMain")?.outcome).isEqualTo(SUCCESS)
        }

        tempFile.writeText("val answer=42\n")

        val result = tempDir.gradle("ktfmtFormatMain").build()

        assertThat(result.task(":ktfmtFormatMain")?.outcome).isEqualTo(SUCCESS)
        assertThat(tempFile.readText()).isEqualTo("val answer = 42\n")
    }

    @Test
    fun `format task prints formatted files with --info`() {
        tempDir.createTempFile(content = "val answer=42\n")
        val result = tempDir.gradle("ktfmtFormatMain", "--info").build()

        assertThat(result.task(":ktfmtFormatMain")?.outcome).isEqualTo(SUCCESS)
        assertThat(result.output).containsMatch("Reformatting .*TestFile.kt")
        assertThat(result.output).contains("Successfully reformatted 1 files with Ktfmt")
    }

    @Test
    fun `format task uses configuration cache correctly`() {
        tempDir.createTempFile(content = "val answer = 42\n")
        tempDir.gradle("--configuration-cache", "--rerun-tasks", "ktfmtFormatMain").build()

        val result =
            tempDir.gradle("--configuration-cache", "--rerun-tasks", "ktfmtFormatMain").build()

        assertThat(result.output).contains("Reusing configuration cache.")
    }

    @Test
    fun `format task reformats all the file even with a failure`() {
        val file1 = tempDir.createTempFile(content = "val answer = `", fileName = "File1.kt")
        val file2 = tempDir.createTempFile(content = "val answer=42", fileName = "File2.kt")

        val result = tempDir.gradle("ktfmtFormatMain", "--info").buildAndFail()

        assertThat(result.task(":ktfmtFormatMain")?.outcome).isEqualTo(FAILED)

        // File 1 contains a parsing error and is untouched.
        assertThat(file1.readText()).isEqualTo("val answer = `")
        assertThat(file2.readText()).isEqualTo("val answer = 42\n")
        assertThat(result.output).containsMatch("Failed to format file: .*File1.kt \\(reason =")
        assertThat(result.output).containsMatch("Reformatting .*File2.kt")
    }

    @Test
    fun `format task runs before compilation`() {
        tempDir.createTempFile(content = "val answer = 42\n")
        val result = tempDir.gradle("compileKotlin", "ktfmtFormatMain", "--dry-run").build()

        assertThat(result.output).contains(":ktfmtFormatMain SKIPPED")
        assertThat(result.output).contains(":compileKotlin SKIPPED")
        assertThat(result.output.indexOf(":ktfmtFormatMain SKIPPED"))
            .isLessThan(result.output.indexOf(":compileKotlin SKIPPED"))
    }

    @Test
    fun `format task skips a file if with --include-only`() {
        tempDir.createTempFile(content = "val answer = `\n", fileName = "File1.kt")
        val file2 = tempDir.createTempFile(content = "val answer=42\n", fileName = "File2.kt")

        val result =
            tempDir
                .gradle("ktfmtFormatMain", "--info", "--include-only=${file2.relativeTo(tempDir)}")
                .build()

        assertThat(result.task(":ktfmtFormatMain")?.outcome).isEqualTo(SUCCESS)
        assertThat(result.output).containsMatch("Reformatting .*File2.kt")
        assertThat(result.output)
            .containsMatch("Skipping format for .*File1.kt because it is not included")
        assertThat(result.output).contains("[ktfmt] Successfully reformatted 1 files with Ktfmt")
    }

    @ParameterizedTest
    @ValueSource(ints = [10, 15, 30, 50, 100, 1000])
    fun `format task can format multiple files`(n: Int) {
        val files =
            List(n) { index ->
                tempDir.createTempFile(
                    content = "val answer${index}=42\n",
                    fileName = "TestFile$index.kt",
                )
            }
        val result = tempDir.gradle("ktfmtFormatMain", "--info").build()

        assertThat(result.task(":ktfmtFormatMain")?.outcome).isEqualTo(SUCCESS)
        files.forEachIndexed { index, file ->
            assertThat(file.readText()).isEqualTo("val answer$index = 42\n")
        }
    }

    @Test
    fun `custom format task should be compatible with configuration cache`() {
        tempDir.createTempFile(content = "val answer = 42\n")

        tempDir.appendToBuildGradle(
            """
            |tasks.register<com.ncorti.ktfmt.gradle.tasks.KtfmtFormatTask>("customFormatTask") {
            |    source = fileTree("src/main/java")
            |}
            """
                .trimMargin()
        )

        tempDir.gradle("customFormatTask", "--configuration-cache").build()
    }

    @Test
    fun `debuggingPrintOpsAfterFormatting logs that ktrs ignores it`() {
        tempDir.createTempFile(content = "val answer = 42\n")
        tempDir.appendToBuildGradle("ktfmt { debuggingPrintOpsAfterFormatting.set(true) }")

        val result = tempDir.gradle("ktfmtFormatMain").build()

        assertThat(result.output)
            .contains("[ktfmt] debuggingPrintOpsAfterFormatting is not supported by ktrs")
    }
}
