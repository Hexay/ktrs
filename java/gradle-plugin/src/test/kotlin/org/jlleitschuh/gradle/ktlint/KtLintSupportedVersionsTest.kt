package org.jlleitschuh.gradle.ktlint

import com.google.common.truth.Truth.assertThat
import org.gradle.testkit.runner.TaskOutcome
import org.jlleitschuh.gradle.ktlint.testdsl.TestProject
import org.jlleitschuh.gradle.ktlint.testdsl.build
import org.jlleitschuh.gradle.ktlint.testdsl.buildAndFail
import org.jlleitschuh.gradle.ktlint.testdsl.project
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.params.ParameterizedTest
import org.junit.jupiter.params.provider.ValueSource

/** Upstream's per-version tests, over the two ktlint versions ktrs runs. */
class KtLintSupportedVersionsTest : AbstractPluginTest() {

    private fun TestProject.useVersion(version: String) {
        buildGradle.appendText("\nktlint.version = \"$version\"\n")
    }

    @DisplayName("Should lint correct sources without errors")
    @ParameterizedTest(name = "KtLint {0}: {displayName}")
    @ValueSource(strings = ["1.8.0", "2.0.0-ALPHA-4"])
    fun lintCleanSources(ktLintVersion: String) {
        project {
            useVersion(ktLintVersion)
            withCleanSources()

            build(CHECK_PARENT_TASK_NAME) {
                assertThat(task(":$mainSourceSetCheckTaskName")?.outcome).isEqualTo(TaskOutcome.SUCCESS)
            }
        }
    }

    @DisplayName("Lint should fail on sources with style violations")
    @ParameterizedTest(name = "KtLint {0}: {displayName}")
    @ValueSource(strings = ["1.8.0", "2.0.0-ALPHA-4"])
    fun lintFailingSources(ktLintVersion: String) {
        project {
            useVersion(ktLintVersion)
            withFailingSources()

            buildAndFail(CHECK_PARENT_TASK_NAME) {
                assertThat(task(":$mainSourceSetCheckTaskName")?.outcome).isEqualTo(TaskOutcome.FAILED)
            }
        }
    }

    @DisplayName("Lint should use editorconfig override (standard rule)")
    @ParameterizedTest(name = "KtLint {0}: {displayName}")
    @ValueSource(strings = ["1.8.0", "2.0.0-ALPHA-4"])
    fun editorconfigOverrideStandardRule(ktLintVersion: String) {
        project {
            useVersion(ktLintVersion)
            buildGradle.appendText("ktlint.additionalEditorconfig = [\"ktlint_standard_no-multi-spaces\": \"disabled\"]\n")
            withFailingSources()

            build(CHECK_PARENT_TASK_NAME) {
                assertThat(task(":$mainSourceSetCheckTaskName")?.outcome).isEqualTo(TaskOutcome.SUCCESS)
            }
        }
    }

    @DisplayName("Lint should use editorconfig override")
    @ParameterizedTest(name = "KtLint {0}: {displayName}")
    @ValueSource(strings = ["1.8.0", "2.0.0-ALPHA-4"])
    fun editorconfigOverride(ktLintVersion: String) {
        project {
            useVersion(ktLintVersion)
            buildGradle.appendText("ktlint.additionalEditorconfig = [\"max_line_length\": \"20\"]\n")
            withFailingMaxLineSources()

            buildAndFail(CHECK_PARENT_TASK_NAME) {
                assertThat(task(":$mainSourceSetCheckTaskName")?.outcome).isEqualTo(TaskOutcome.FAILED)
                // Upstream adds " (cannot be auto-corrected)"; check-task console rows here don't (research/29).
                assertThat(output).contains("Exceeded max line length (20)")
            }
        }
    }

    @DisplayName("Format should successfully finish on sources with style violations")
    @ParameterizedTest(name = "KtLint {0}: {displayName}")
    @ValueSource(strings = ["1.8.0", "2.0.0-ALPHA-4"])
    fun formatFailingSources(ktLintVersion: String) {
        project {
            useVersion(ktLintVersion)
            withFailingSources()

            build(FORMAT_PARENT_TASK_NAME) {
                assertThat(task(":$mainSourceSetFormatTaskName")?.outcome).isEqualTo(TaskOutcome.SUCCESS)
            }
        }
    }

    @DisplayName("Should lint without errors when 'final-newline' rule is disabled via editorconfig")
    @ParameterizedTest(name = "KtLint {0}: {displayName}")
    @ValueSource(strings = ["1.8.0", "2.0.0-ALPHA-4"])
    fun lintDisabledRuleFinalNewlineEditorconfig(ktLintVersion: String) {
        project {
            editorConfig.appendText("\nroot = true\n\n[*.kt]\nktlint_standard_final-newline = disabled\n")
            useVersion(ktLintVersion)
            createSourceFile("src/main/kotlin/CleanSource.kt", "val foo = \"bar\"")

            build(CHECK_PARENT_TASK_NAME) {
                assertThat(task(":$mainSourceSetCheckTaskName")?.outcome).isEqualTo(TaskOutcome.SUCCESS)
                assertThat(output).doesNotContain("Property 'ktlint_disabled_rules' is deprecated")
                assertThat(output).doesNotContain("Property 'disabled_rules' is deprecated")
            }
        }
    }
}
