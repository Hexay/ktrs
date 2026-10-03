package org.jlleitschuh.gradle.ktlint

import com.google.common.truth.Truth.assertThat
import org.gradle.testkit.runner.TaskOutcome
import org.jlleitschuh.gradle.ktlint.tasks.GenerateReportsTask
import org.jlleitschuh.gradle.ktlint.testdsl.TestProject
import org.jlleitschuh.gradle.ktlint.testdsl.build
import org.jlleitschuh.gradle.ktlint.testdsl.project
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.Test

/** Upstream's BuildCacheTest, without the 3rd party reporter case (it downloads the reporter). */
class BuildCacheTest : AbstractPluginTest() {
    private val originalRoot get() = temporaryFolder.resolve("original").apply { mkdirs() }
    private val relocatedRoot get() = temporaryFolder.resolve("relocated").apply { mkdirs() }
    private val localBuildCache get() = temporaryFolder.resolve("build-cache").apply { mkdirs() }

    @DisplayName("Check task should be relocatable")
    @Test
    fun checkIsRelocatable() {
        val testSourceCheckTaskName =
            GenerateReportsTask.generateNameForSourceSets("test", GenerateReportsTask.LintType.CHECK)
        project(projectPath = originalRoot) {
            configureDefaultProject()

            build(CHECK_PARENT_TASK_NAME, "--build-cache") {
                assertThat(task(":$mainSourceSetCheckTaskName")?.outcome).isEqualTo(TaskOutcome.SUCCESS)
                assertThat(task(":$testSourceCheckTaskName")?.outcome).isEqualTo(TaskOutcome.SUCCESS)
            }
        }

        project(projectPath = relocatedRoot) {
            configureDefaultProject()

            build(CHECK_PARENT_TASK_NAME, "--build-cache") {
                assertThat(task(":$mainSourceSetCheckTaskName")?.outcome).isEqualTo(TaskOutcome.FROM_CACHE)
                assertThat(task(":$testSourceCheckTaskName")?.outcome).isEqualTo(TaskOutcome.FROM_CACHE)
            }
        }
    }

    private fun TestProject.configureDefaultProject() {
        settingsGradle.appendText("\nbuildCache {\n    local {\n        directory = '${localBuildCache.toURI()}'\n    }\n}\n")
        withCleanSources()
        createSourceFile("src/test/kotlin/Test.kt", "class Test\n")
    }
}
