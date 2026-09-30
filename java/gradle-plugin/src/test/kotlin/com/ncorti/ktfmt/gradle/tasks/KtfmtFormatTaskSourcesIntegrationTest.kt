package com.ncorti.ktfmt.gradle.tasks

import com.google.common.truth.Truth.assertThat
import com.ncorti.ktfmt.gradle.testutil.appendToBuildGradle
import com.ncorti.ktfmt.gradle.testutil.copyFixture
import com.ncorti.ktfmt.gradle.testutil.createTempFile
import com.ncorti.ktfmt.gradle.testutil.gradle
import java.io.File
import org.gradle.testkit.runner.TaskOutcome.FAILED
import org.gradle.testkit.runner.TaskOutcome.NO_SOURCE
import org.gradle.testkit.runner.TaskOutcome.SUCCESS
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir

/** The second half of ktfmt-gradle's `KtfmtFormatTaskIntegrationTest`. */
internal class KtfmtFormatTaskSourcesIntegrationTest {

    @TempDir lateinit var tempDir: File

    @BeforeEach
    fun setUp() {
        tempDir.copyFixture("jvmProject")
    }

    private val unformatted =
        """
        |fun someFun(){
        |println("Hello, World!")
        |println("HelloWorld2")
        |}
        """
            .trimMargin()

    @Test
    fun `should format the files in kotlinLang style with a 4 space indentation`() {
        val file = tempDir.createTempFile(content = unformatted)
        tempDir.appendToBuildGradle("ktfmt { kotlinLangStyle() }")

        tempDir.gradle("ktfmtFormatMain").build()

        assertThat(file.readLines())
            .containsExactly(
                "fun someFun() {",
                "    println(\"Hello, World!\")",
                "    println(\"HelloWorld2\")",
                "}",
            )
    }

    @Test
    fun `should format the files in googleStyle style with a 2 space indentation`() {
        val file = tempDir.createTempFile(content = unformatted)
        tempDir.appendToBuildGradle("ktfmt { googleStyle() }")

        tempDir.gradle("ktfmtFormatMain").build()

        assertThat(file.readLines())
            .containsExactly(
                "fun someFun() {",
                "  println(\"Hello, World!\")",
                "  println(\"HelloWorld2\")",
                "}",
            )
    }

    @Test
    fun `explicit options override the style`() {
        val file =
            tempDir.createTempFile(content = "import a.B\n\nfun someFun(){\nprintln(1)\n}\n")
        tempDir.appendToBuildGradle(
            """
            |ktfmt {
            |    blockIndent.set(3)
            |    removeUnusedImports.set(false)
            |}
            """
                .trimMargin()
        )

        tempDir.gradle("ktfmtFormatMain").build()

        assertThat(file.readText()).isEqualTo("import a.B\n\nfun someFun() {\n   println(1)\n}\n")
    }

    @Test
    fun `format task should detect the source and test files in a flattened project structure and format them`() {
        tempDir.appendToBuildGradle(
            """
            |kotlin {
            |    sourceSets.main { kotlin.setSrcDirs(listOf("src")) }
            |    sourceSets.test { kotlin.setSrcDirs(listOf("test")) }
            |}
            """
                .trimMargin()
        )

        val sourceFile = tempDir.createTempFile("val answer =  42\n", path = "src/someFolder")
        val testFile = tempDir.createTempFile("val answer =  42\n", path = "test/someOtherFolder")

        val result = tempDir.gradle("ktfmtFormat").build()

        assertThat(result.task(":ktfmtFormatMain")?.outcome).isNotEqualTo(NO_SOURCE)
        assertThat(result.task(":ktfmtFormatTest")?.outcome).isNotEqualTo(NO_SOURCE)

        assertThat(sourceFile.readText()).contains("val answer = 42\n")
        assertThat(testFile.readText()).contains("val answer = 42\n")
    }

    @Test
    fun `format task should by default not format sourceSets in the build folder`() {
        tempDir.appendToBuildGradle("kotlin { sourceSets.main { kotlin.srcDirs(\"build/main\") } }")
        val file = tempDir.createTempFile(content = "val answer=42\n", path = "build/main")

        val result = tempDir.gradle("ktfmtFormat").build()

        assertThat(file.readText()).isEqualTo("val answer=42\n")
        assertThat(result.task(":ktfmtFormatMain")?.outcome).isEqualTo(NO_SOURCE)
    }

    @Test
    fun `format task should not ignore sourceSets in build folder when a custom exclusion pattern is specified`() {
        tempDir.appendToBuildGradle(
            """
            |kotlin { sourceSets.main { kotlin.srcDirs("build/generated") } }
            |
            |ktfmt { srcSetPathExclusionPattern.set(Regex("customRules.*")) }
            """
                .trimMargin()
        )

        val file =
            tempDir.createTempFile(content = "val answer=42\n", path = "build/generated/main")

        tempDir.gradle("ktfmtFormat", "--info").build()

        assertThat(file.readText()).isEqualTo("val answer = 42\n")
    }

    @Test
    fun `format task should ignore the main sourceSets when specified as exclusion pattern`() {
        tempDir.appendToBuildGradle(
            "ktfmt { srcSetPathExclusionPattern.set(Regex(\".*[\\\\\\\\/]main[\\\\\\\\/].*\")) }"
        )
        tempDir.createTempFile(content = "val answer=42\n")

        val result = tempDir.gradle("ktfmtFormat").build()

        assertThat(result.task(":ktfmtFormatMain")?.outcome).isEqualTo(NO_SOURCE)
    }

    @Test
    fun `format scripts task should fail if top-level script file could not be parsed`() {
        val scriptFile =
            tempDir.createTempFile(content = "val answer=\n", fileName = "my.kts", path = "")

        val result = tempDir.gradle("ktfmtFormatScripts").buildAndFail()

        assertThat(scriptFile.readText()).isEqualTo("val answer=\n")
        assertThat(result.task(":ktfmtFormatScripts")?.outcome).isEqualTo(FAILED)
    }

    @Test
    fun `format scripts task should format top-level script file`() {
        val scriptFile =
            tempDir.createTempFile(content = "val answer=42\n", fileName = "my.kts", path = "")

        val result = tempDir.gradle("ktfmtFormatScripts").build()

        assertThat(scriptFile.readText()).isEqualTo("val answer = 42\n")
        assertThat(result.task(":ktfmtFormatScripts")?.outcome).isEqualTo(SUCCESS)
    }

    @Test
    fun `format scripts task should not format non top-level script file`() {
        val scriptFile = tempDir.createTempFile(content = "val answer=42\n", fileName = "my.kts")

        val result = tempDir.gradle("ktfmtFormatScripts").build()

        assertThat(scriptFile.readText()).isEqualTo("val answer=42\n")
        assertThat(result.task(":ktfmtFormatScripts")?.outcome).isEqualTo(SUCCESS)
    }

    @Test
    fun `format scripts task should format top-level script file on project without any kotlin plugins`() {
        val scriptFile = tempDir.resolve("root-script.kts")
        scriptFile.writeText("val x=1\n")

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

        val result = tempDir.gradle("ktfmtFormatScripts").build()

        assertThat(result.task(":ktfmtFormatScripts")?.outcome).isEqualTo(SUCCESS)
        assertThat(scriptFile.readText()).isEqualTo("val x = 1\n")
    }
}
