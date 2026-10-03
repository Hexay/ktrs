package org.jlleitschuh.gradle.ktlint

import com.google.common.truth.Truth.assertThat
import java.io.File
import org.jlleitschuh.gradle.ktlint.tasks.GenerateReportsTask
import org.jlleitschuh.gradle.ktlint.tasks.GenerateReportsTask.LintType
import org.jlleitschuh.gradle.ktlint.testdsl.build
import org.jlleitschuh.gradle.ktlint.testdsl.project
import org.jlleitschuh.gradle.ktlint.testdsl.projectSetup
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.Test

/**
 * Upstream's KotlinMultiplatformPluginTests. Its KotlinJsPluginTests are not ported: Kotlin 2.4
 * fails the build when `org.jetbrains.kotlin.js` is applied.
 */
class KotlinPluginsTests : AbstractPluginTest() {
    private fun multiplatformProjectSetup(): (File) -> Unit = {
        projectSetup("multiplatform").invoke(it)
        it.resolve("build.gradle").appendText("\nkotlin {\n    js {\n      browser()\n    }\n    jvm()\n}\n")
    }

    @DisplayName("Should add check on all sources")
    @Test
    fun addCheckTasks() {
        project(projectSetup = multiplatformProjectSetup()) {
            build("-m", CHECK_PARENT_TASK_NAME) {
                val lines = output.lineSequence().toList()
                for (sourceSet in listOf("commonMain", "JsMain", "JvmMain")) {
                    val name = GenerateReportsTask.generateNameForSourceSets(sourceSet, LintType.CHECK)
                    assertThat(lines.any { it.contains(name) }).isTrue()
                }
            }
        }
    }
}
