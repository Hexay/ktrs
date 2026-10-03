package org.jlleitschuh.gradle.ktlint

import com.google.common.truth.Truth.assertThat
import org.gradle.testkit.runner.TaskOutcome
import org.jlleitschuh.gradle.ktlint.tasks.KtLintFormatTask
import org.jlleitschuh.gradle.ktlint.testdsl.build
import org.jlleitschuh.gradle.ktlint.testdsl.project
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.Test
import org.junit.jupiter.params.ParameterizedTest
import org.junit.jupiter.params.provider.ValueSource

class ConfigurationCacheTest : AbstractPluginTest() {
    private val configurationCacheFlag = "--configuration-cache"
    private val formatTaskName = KtLintFormatTask.buildTaskNameForSourceSet("main")

    @DisplayName("Should support configuration cache without errors on running linting")
    @Test
    fun configurationCacheForCheckTask() {
        project {
            createSourceFile("src/main/kotlin/CleanSource.kt", "val foo = \"bar\"\n")

            build(configurationCacheFlag, CHECK_PARENT_TASK_NAME) {
                assertThat(task(":$mainSourceSetCheckTaskName")?.outcome).isEqualTo(TaskOutcome.SUCCESS)
            }
            build(configurationCacheFlag, CHECK_PARENT_TASK_NAME) {
                assertThat(task(":$mainSourceSetCheckTaskName")?.outcome).isEqualTo(TaskOutcome.UP_TO_DATE)
                assertThat(output).contains("Reusing configuration cache.")
            }
        }
    }

    @DisplayName("Should support configuration cache on running format tasks")
    @Test
    fun configurationCacheForFormatTasks() {
        project {
            withCleanSources()
            build(configurationCacheFlag, FORMAT_PARENT_TASK_NAME) {
                assertThat(task(":$formatTaskName")?.outcome).isEqualTo(TaskOutcome.SUCCESS)
                assertThat(task(":$mainSourceSetFormatTaskName")?.outcome).isEqualTo(TaskOutcome.SUCCESS)
            }
            build(configurationCacheFlag, FORMAT_PARENT_TASK_NAME) {
                assertThat(task(":$formatTaskName")?.outcome).isEqualTo(TaskOutcome.UP_TO_DATE)
                assertThat(task(":$mainSourceSetFormatTaskName")?.outcome).isEqualTo(TaskOutcome.UP_TO_DATE)
                assertThat(output).contains("Reusing configuration cache.")
            }
        }
    }

    @DisplayName("Should support configuration cache on running format tasks with relative paths")
    @Test
    fun configurationCacheForFormatTasksWithRelativePaths() {
        project {
            buildGradle.appendText(
                "ktlint {\n    relative = true\n    reporters {\n        reporter \"plain\"\n        reporter \"checkstyle\"\n    }\n}\n"
            )
            withCleanSources()
            build(configurationCacheFlag, FORMAT_PARENT_TASK_NAME) {
                assertThat(task(":$formatTaskName")?.outcome).isEqualTo(TaskOutcome.SUCCESS)
                assertThat(task(":$mainSourceSetFormatTaskName")?.outcome).isEqualTo(TaskOutcome.SUCCESS)
            }
            build(configurationCacheFlag, FORMAT_PARENT_TASK_NAME) {
                assertThat(task(":$formatTaskName")?.outcome).isEqualTo(TaskOutcome.UP_TO_DATE)
                assertThat(task(":$mainSourceSetFormatTaskName")?.outcome).isEqualTo(TaskOutcome.UP_TO_DATE)
                assertThat(output).contains("Reusing configuration cache.")
            }
        }
    }

    @DisplayName("Should support configuration cache for git hook install tasks")
    @ParameterizedTest(name = "{0}: {displayName}")
    @ValueSource(strings = [INSTALL_GIT_HOOK_FORMAT_TASK, INSTALL_GIT_HOOK_CHECK_TASK])
    fun configurationCacheForGitHookInstallTask(taskName: String) {
        project {
            projectPath.initGit()

            build(configurationCacheFlag, taskName) {
                assertThat(task(":$taskName")?.outcome).isEqualTo(TaskOutcome.SUCCESS)
            }
            build(configurationCacheFlag, taskName) {
                assertThat(task(":$taskName")?.outcome).isEqualTo(TaskOutcome.SUCCESS)
                assertThat(output).contains("Reusing configuration cache.")
            }
        }
    }
}
