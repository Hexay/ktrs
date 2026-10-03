package org.jlleitschuh.gradle.ktlint

import com.google.common.truth.Truth.assertThat
import io.github.hexay.ktrs.gradle.ktlint.KtlintVersions
import org.gradle.testkit.runner.TaskOutcome
import org.jlleitschuh.gradle.ktlint.tasks.LoadReportersTask
import org.jlleitschuh.gradle.ktlint.testdsl.build
import org.jlleitschuh.gradle.ktlint.testdsl.buildAndFail
import org.jlleitschuh.gradle.ktlint.testdsl.project
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.Test

/** Upstream's KtLint version tests, for the two versions ktrs runs. */
class KtlintPluginVersionTest : AbstractPluginTest() {

    @DisplayName("Should apply KtLint version from extension")
    @Test
    fun ktlintVersionFromExtension() {
        project {
            withFailingSources()
            buildGradle.appendText(
                """
                ktlint.version = "${KtlintVersions.V2_0}"
                ktlint.debug = true
                """
                    .trimIndent()
            )

            buildAndFail(CHECK_PARENT_TASK_NAME) {
                assertThat(task(":$mainSourceSetCheckTaskName")?.outcome).isEqualTo(TaskOutcome.FAILED)
                assertThat(output).contains("--ktlint-version=2.0")
            }
        }
    }

    @DisplayName("Should default to KtLint 1.8.0")
    @Test
    fun defaultKtlintVersion() {
        project {
            withCleanSources()
            buildGradle.appendText("ktlint.debug = true\n")

            build(CHECK_PARENT_TASK_NAME) {
                assertThat(task(":$mainSourceSetCheckTaskName")?.outcome).isEqualTo(TaskOutcome.SUCCESS)
                assertThat(output).contains("--ktlint-version=1.8")
            }
        }
    }

    @DisplayName("Should apply KtLint version from ktlint-plugins.properties")
    @Test
    fun `ktlint version from properties`() {
        project {
            withCleanSources()
            val propsFile = projectPath.resolve("ktlint-plugins.properties")
            propsFile.writeText("ktlint-version=${KtlintVersions.V2_0}")

            build(":$CHECK_PARENT_TASK_NAME", "--warning-mode=all") {
                assertThat(task(":$mainSourceSetCheckTaskName")?.outcome).isEqualTo(TaskOutcome.SUCCESS)
                assertThat(output).doesNotContain("has been deprecated")
            }
            build(":$CHECK_PARENT_TASK_NAME") {
                assertThat(task(":$mainSourceSetCheckTaskName")?.outcome).isEqualTo(TaskOutcome.UP_TO_DATE)
            }
            propsFile.writeText("ktlint-version=${KtlintVersions.V1_8}")
            build(":$CHECK_PARENT_TASK_NAME") {
                assertThat(task(":${KtlintPluginVersionTest.MAIN_LINT_TASK}")?.outcome).isEqualTo(TaskOutcome.SUCCESS)
            }
        }
    }

    @DisplayName("Should fail the build on a KtLint version ktrs does not run")
    @Test
    fun failOnUnsupportedKtLintVersion() {
        project {
            withCleanSources()
            buildGradle.appendText("ktlint.version = \"1.5.0\"\n")

            buildAndFail(CHECK_PARENT_TASK_NAME) {
                assertThat(task(":${LoadReportersTask.TASK_NAME}")?.outcome).isEqualTo(TaskOutcome.FAILED)
                assertThat(output).contains("ktrs runs ktlint 1.8.0 or 2.0.0-ALPHA-4, not ktlint 1.5.0")
            }
        }
    }

    private companion object {
        const val MAIN_LINT_TASK = "runKtlintCheckOverMainSourceSet"
    }
}
