package org.jlleitschuh.gradle.ktlint

import com.google.common.truth.Truth.assertThat
import org.gradle.testkit.runner.TaskOutcome
import org.jlleitschuh.gradle.ktlint.tasks.KtLintCheckTask
import org.jlleitschuh.gradle.ktlint.testdsl.PLUGIN_ID
import org.jlleitschuh.gradle.ktlint.testdsl.TestProject
import org.jlleitschuh.gradle.ktlint.testdsl.build
import org.jlleitschuh.gradle.ktlint.testdsl.buildAndFail
import org.jlleitschuh.gradle.ktlint.testdsl.project
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.Test

/** `.editorconfig` support. */
class EditorConfigTests : AbstractPluginTest() {
    private val lintTaskName = KtLintCheckTask.buildTaskNameForSourceSet("main")

    @DisplayName("Check task should be UP-TO-DATE if '.editorconfig' content didn't change")
    @Test
    fun checkUpToDateOnSameContent() {
        project {
            withCleanSources()
            createEditorconfigFile()

            build(CHECK_PARENT_TASK_NAME) {
                assertThat(task(":$lintTaskName")?.outcome).isEqualTo(TaskOutcome.SUCCESS)
            }
            build(CHECK_PARENT_TASK_NAME) {
                assertThat(task(":$lintTaskName")?.outcome).isEqualTo(TaskOutcome.UP_TO_DATE)
            }
        }
    }

    @DisplayName("Check task should rerun if '.editorconfig' content has changed")
    @Test
    fun checkRerunOnContentChange() {
        project {
            withCleanSources()
            createEditorconfigFile()

            build(CHECK_PARENT_TASK_NAME) {
                assertThat(task(":$lintTaskName")?.outcome).isEqualTo(TaskOutcome.SUCCESS)
            }

            modifyEditorconfigFile(10)
            buildAndFail(CHECK_PARENT_TASK_NAME) {
                assertThat(task(":$lintTaskName")?.outcome).isEqualTo(TaskOutcome.SUCCESS)
                assertThat(task(":$mainSourceSetCheckTaskName")?.outcome).isEqualTo(TaskOutcome.FAILED)
            }
        }
    }

    @DisplayName("Check task should rerun if root '.editorconfig' file content has changed")
    @Test
    fun checkRerunOnRootFileContentChange() {
        val projectWithModulesLocation = temporaryFolder.resolve("modularized").also { it.mkdirs() }
        project(projectPath = projectWithModulesLocation) {
            val moduleLocation = projectWithModulesLocation.resolve("test/module1").also { it.mkdirs() }
            settingsGradle.appendText("\ninclude \":test:module1\"\n")
            buildGradle.appendText("\nallprojects {\n    repositories {\n        mavenCentral()\n    }\n}\n")
            createEditorconfigFile()
            moduleLocation.buildFile().writeText(
                """
                plugins {
                    id "org.jetbrains.kotlin.jvm"
                    id "$PLUGIN_ID"
                }
                """
                    .trimIndent()
            )
            moduleLocation.withCleanSources()

            build(":test:module1:$CHECK_PARENT_TASK_NAME") {
                assertThat(task(":test:module1:$lintTaskName")?.outcome).isEqualTo(TaskOutcome.SUCCESS)
            }

            modifyEditorconfigFile(10)

            buildAndFail(":test:module1:$CHECK_PARENT_TASK_NAME") {
                assertThat(task(":test:module1:$lintTaskName")?.outcome).isEqualTo(TaskOutcome.SUCCESS)
                assertThat(task(":test:module1:$mainSourceSetCheckTaskName")?.outcome).isEqualTo(TaskOutcome.FAILED)
            }
        }
    }

    @DisplayName("Check task should rerun if additionalEditorconfig property changes")
    @Test
    fun checkRerunOnAdditionalEditorconfigPropertyChange() {
        project {
            withAdditionalEditorconfigProperty(120)
            withCleanSources()

            build(CHECK_PARENT_TASK_NAME) {
                assertThat(task(":$lintTaskName")?.outcome).isEqualTo(TaskOutcome.SUCCESS)
            }

            withAdditionalEditorconfigProperty(10)

            buildAndFail(CHECK_PARENT_TASK_NAME) {
                assertThat(task(":$lintTaskName")?.outcome).isEqualTo(TaskOutcome.SUCCESS)
                assertThat(task(":$mainSourceSetCheckTaskName")?.outcome).isEqualTo(TaskOutcome.FAILED)
            }
        }
    }

    private fun TestProject.createEditorconfigFile(maxLineLength: Int = 120) =
        createSourceFile(".editorconfig", "[*.{kt,kts}]\nmax_line_length=$maxLineLength")

    private fun TestProject.modifyEditorconfigFile(maxLineLength: Int) {
        projectPath.resolve(".editorconfig").delete()
        createEditorconfigFile(maxLineLength)
    }

    private fun TestProject.withAdditionalEditorconfigProperty(maxLineLength: Int) {
        buildGradle.appendText("\nktlint {\n    additionalEditorconfig[\"max_line_length\"] = \"$maxLineLength\"\n}\n")
    }
}
