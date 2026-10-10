package org.jmailen.gradle.kotlinter.functional

import java.io.File
import org.gradle.testkit.runner.TaskOutcome.FAILED
import org.gradle.testkit.runner.TaskOutcome.SUCCESS
import org.gradle.testkit.runner.TaskOutcome.UP_TO_DATE
import org.jmailen.gradle.kotlinter.functional.utils.PLUGIN_ID
import org.jmailen.gradle.kotlinter.functional.utils.repositories
import org.jmailen.gradle.kotlinter.functional.utils.resolve
import org.jmailen.gradle.kotlinter.functional.utils.settingsFile
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test

/** Not upstream's: `kotlinter { ktlintVersion }` selects one of the two ktlint versions ktrs runs (research/34, D4). */
internal class KtlintVersionTest : WithGradleTest.Kotlin() {

    private lateinit var projectRoot: File

    @BeforeEach
    fun setUp() {
        projectRoot =
            testProjectDir.apply {
                resolve("settings.gradle") { writeText(settingsFile) }
                resolve("src/main/kotlin/CleanSource.kt") { writeText("val foo = \"bar\"\n") }
            }
    }

    @Test
    fun `lintKotlinMain and formatKotlinMain fail for a ktlint version ktrs does not run`() {
        buildFile(ktlintVersion = "1.5.0")

        buildAndFail("lintKotlinMain").apply {
            assertEquals(FAILED, task(":lintKotlinMain")?.outcome)
            assertUnsupportedVersionMessage(output)
        }
        buildAndFail("formatKotlinMain").apply {
            assertEquals(FAILED, task(":formatKotlinMain")?.outcome)
            assertUnsupportedVersionMessage(output)
        }
    }

    @Test
    fun `lintKotlinMain succeeds with ktlint 2_0_0-ALPHA-4`() {
        buildFile(ktlintVersion = "2.0.0-ALPHA-4")

        build("lintKotlinMain").apply { assertEquals(SUCCESS, task(":lintKotlinMain")?.outcome) }
    }

    @Test
    fun `lintKotlinMain reruns when ktlintVersion changes`() {
        buildFile(ktlintVersion = "1.8.0")
        build("lintKotlinMain").apply { assertEquals(SUCCESS, task(":lintKotlinMain")?.outcome) }
        build("lintKotlinMain").apply { assertEquals(UP_TO_DATE, task(":lintKotlinMain")?.outcome) }

        buildFile(ktlintVersion = "2.0.0-ALPHA-4")
        build("lintKotlinMain").apply { assertEquals(SUCCESS, task(":lintKotlinMain")?.outcome) }
        build("lintKotlinMain").apply { assertEquals(UP_TO_DATE, task(":lintKotlinMain")?.outcome) }
    }

    private fun assertUnsupportedVersionMessage(output: String) {
        assertTrue(output.contains("ktrs runs ktlint 1.8.0 or 2.0.0-ALPHA-4, not ktlint 1.5.0"))
        assertTrue(output.contains("set `kotlinter { ktlintVersion }` to one of them"))
    }

    private fun buildFile(ktlintVersion: String) {
        projectRoot.resolve("build.gradle") {
            val buildScript =
                """
                plugins {
                    id 'org.jetbrains.kotlin.jvm'
                    id '$PLUGIN_ID'
                }
                $repositories

                kotlinter {
                    ktlintVersion = '$ktlintVersion'
                }
                """.trimIndent()
            writeText(buildScript)
        }
    }
}
