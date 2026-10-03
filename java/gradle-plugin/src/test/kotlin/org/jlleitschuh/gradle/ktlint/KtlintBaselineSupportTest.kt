package org.jlleitschuh.gradle.ktlint

import com.google.common.truth.Truth.assertThat
import java.io.File
import org.gradle.testkit.runner.TaskOutcome
import org.jlleitschuh.gradle.ktlint.tasks.GenerateBaselineTask
import org.jlleitschuh.gradle.ktlint.testdsl.TestProject.Companion.FAIL_SOURCE_FILE
import org.jlleitschuh.gradle.ktlint.testdsl.build
import org.jlleitschuh.gradle.ktlint.testdsl.buildAndFail
import org.jlleitschuh.gradle.ktlint.testdsl.project
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.Test
import org.junit.jupiter.params.ParameterizedTest
import org.junit.jupiter.params.provider.ValueSource

class KtlintBaselineSupportTest : AbstractPluginTest() {

    private val emptyBaseline = "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<baseline version=\"1.0\">\n</baseline>\n"

    @DisplayName("Should generate empty baseline file on style violations")
    @Test
    fun emptyBaseline() {
        project {
            withCleanSources()

            build(GenerateBaselineTask.NAME) {
                assertThat(task(":${GenerateBaselineTask.NAME}")?.outcome).isEqualTo(TaskOutcome.SUCCESS)
                assertThat(projectPath.defaultBaselineFile.readText().normalizeNewlines()).isEqualTo(emptyBaseline)
            }
        }
    }

    @DisplayName("Should generate baseline with found style violations")
    @Test
    fun generateBaseline() {
        project {
            withFailingSources()
            withFailingKotlinScript()

            build(GenerateBaselineTask.NAME) {
                assertThat(task(":${GenerateBaselineTask.NAME}")?.outcome).isEqualTo(TaskOutcome.SUCCESS)
                // The baseline indents with tabs.
                val expected =
                    """
                    |<?xml version="1.0" encoding="utf-8"?>
                    |<baseline version="1.0">
                    |	<file name="kotlin-script-fail.kts">
                    |		<error line="1" column="15" source="standard:no-trailing-spaces" />
                    |	</file>
                    |	<file name="src/main/kotlin/FailSource.kt">
                    |		<error line="1" column="5" source="standard:no-multi-spaces" />
                    |		<error line="1" column="10" source="standard:no-multi-spaces" />
                    |		<error line="1" column="15" source="standard:no-multi-spaces" />
                    |	</file>
                    |</baseline>
                    |
                    """
                        .trimMargin()
                assertThat(projectPath.defaultBaselineFile.readText().withoutWhitespace())
                    .isEqualTo(expected.withoutWhitespace())
            }
        }
    }

    @DisplayName("Should overwrite existing baseline file")
    @Test
    fun overwriteBaselineFile() {
        project {
            withFailingSources()

            build(GenerateBaselineTask.NAME)

            removeSourceFile(FAIL_SOURCE_FILE)

            build(GenerateBaselineTask.NAME) {
                assertThat(task(":${GenerateBaselineTask.NAME}")?.outcome).isEqualTo(TaskOutcome.SUCCESS)
                assertThat(projectPath.defaultBaselineFile.readText().normalizeNewlines()).isEqualTo(emptyBaseline)
            }
        }
    }

    @DisplayName("Should consider existing issues in baseline")
    @ParameterizedTest(name = "KtLint {0}: {displayName}")
    @ValueSource(strings = ["1.8.0", "2.0.0-ALPHA-4"])
    fun existingIssueFilteredByBaseline(ktLintVersion: String) {
        project {
            buildGradle.appendText("ktlint.version = \"$ktLintVersion\"\nktlint.debug = true\n")
            withFailingSources()

            build(GenerateBaselineTask.NAME)
            build(CHECK_PARENT_TASK_NAME) {
                assertThat(task(":$mainSourceSetCheckTaskName")?.outcome).isEqualTo(TaskOutcome.SUCCESS)
            }
        }
    }

    @DisplayName("Check task should still fail on file style violation that is not present in the baseline")
    @Test
    fun failOnNewStyleViolation() {
        project {
            withFailingSources()
            build(GenerateBaselineTask.NAME)

            withFailingKotlinScript()
            buildAndFail(CHECK_PARENT_TASK_NAME) {
                assertThat(task(":$kotlinScriptCheckTaskName")?.outcome).isEqualTo(TaskOutcome.FAILED)
            }
        }
    }

    private val File.defaultBaselineFile
        get() = resolve("config").resolve("ktlint").resolve("baseline.xml")

    private fun String.normalizeNewlines() = replace("\r\n", "\n")

    private fun String.withoutWhitespace() = filterNot { it.isWhitespace() }
}
