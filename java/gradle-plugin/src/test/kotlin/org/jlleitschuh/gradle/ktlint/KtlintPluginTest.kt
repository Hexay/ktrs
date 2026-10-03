package org.jlleitschuh.gradle.ktlint

import com.google.common.truth.Truth.assertThat
import java.io.File
import org.gradle.testkit.runner.TaskOutcome
import org.jlleitschuh.gradle.ktlint.tasks.GenerateBaselineTask
import org.jlleitschuh.gradle.ktlint.tasks.GenerateReportsTask
import org.jlleitschuh.gradle.ktlint.tasks.KtLintFormatTask
import org.jlleitschuh.gradle.ktlint.testdsl.PLUGIN_ID
import org.jlleitschuh.gradle.ktlint.testdsl.TestProject.Companion.FAIL_SOURCE_FILE
import org.jlleitschuh.gradle.ktlint.testdsl.build
import org.jlleitschuh.gradle.ktlint.testdsl.buildAndFail
import org.jlleitschuh.gradle.ktlint.testdsl.project
import org.jlleitschuh.gradle.ktlint.testdsl.projectSetup
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.Test

class KtlintPluginTest : AbstractPluginTest() {

    @DisplayName("Should fail on failing sources")
    @Test
    fun failOnStyleViolation() {
        project {
            withFailingSources()

            buildAndFail(CHECK_PARENT_TASK_NAME) {
                assertThat(task(":$mainSourceSetCheckTaskName")?.outcome).isEqualTo(TaskOutcome.FAILED)
                assertThat(output).contains("Unnecessary long whitespace")
            }
        }
    }

    @DisplayName("Should succeed check on clean sources")
    @Test
    fun passCleanSources() {
        project {
            withCleanSources()

            build(CHECK_PARENT_TASK_NAME) {
                assertThat(task(":$mainSourceSetCheckTaskName")?.outcome).isEqualTo(TaskOutcome.SUCCESS)
            }
        }
    }

    @DisplayName("Should work with project isolation")
    @Test
    fun `work with project isolation`() {
        project {
            withCleanSources()

            build(CHECK_PARENT_TASK_NAME, "-Dorg.gradle.unsafe.isolated-projects=true") {
                assertThat(task(":$mainSourceSetCheckTaskName")?.outcome).isEqualTo(TaskOutcome.SUCCESS)
            }
        }
    }

    @DisplayName("Should show only plugin meta tasks in task output")
    @Test
    fun showOnlyMetaTasks() {
        project {
            withCleanSources()

            build("tasks") {
                val ktlintTasks = output.lineSequence().filter { it.startsWith("ktlint", ignoreCase = true) }.toList()

                assertThat(ktlintTasks).hasSize(3)
                assertThat(ktlintTasks.any { it.startsWith(CHECK_PARENT_TASK_NAME) }).isTrue()
                assertThat(ktlintTasks.any { it.startsWith(FORMAT_PARENT_TASK_NAME) }).isTrue()
                assertThat(ktlintTasks.any { it.startsWith(GenerateBaselineTask.NAME) }).isTrue()
            }
        }
    }

    @DisplayName("Should show all KtLint tasks in task output")
    @Test
    fun allKtlintTasks() {
        project {
            build("tasks", "--all") {
                val ktlintTasks = output.lineSequence().filter { it.startsWith("ktlint", ignoreCase = true) }.toList()

                // Main and test sources format and check tasks, two kotlin script tasks, meta tasks, baseline.
                assertThat(ktlintTasks).hasSize(9)
                assertThat(ktlintTasks.any { it.startsWith(CHECK_PARENT_TASK_NAME) }).isTrue()
                assertThat(ktlintTasks.any { it.startsWith(FORMAT_PARENT_TASK_NAME) }).isTrue()
                assertThat(ktlintTasks.any { it.startsWith(kotlinScriptCheckTaskName) }).isTrue()
                val scriptFormat = GenerateReportsTask.generateNameForKotlinScripts(GenerateReportsTask.LintType.FORMAT)
                assertThat(ktlintTasks.any { it.startsWith(scriptFormat) }).isTrue()
                assertThat(ktlintTasks.any { it.startsWith(GenerateBaselineTask.NAME) }).isTrue()
            }
        }
    }

    @DisplayName("Should always format again restored to pre-format state sources")
    @Test
    fun repeatFormatForRestoredSources() {
        project {
            withFailingSources()
            val formatTaskName = KtLintFormatTask.buildTaskNameForSourceSet("main")

            build(FORMAT_PARENT_TASK_NAME) {
                assertThat(task(":$formatTaskName")?.outcome).isEqualTo(TaskOutcome.SUCCESS)
            }

            restoreFailingSources()

            build(FORMAT_PARENT_TASK_NAME) {
                assertThat(task(":$formatTaskName")?.outcome).isEqualTo(TaskOutcome.SUCCESS)
                assertThat(projectPath.resolve(FAIL_SOURCE_FILE).exists()).isTrue()
            }

            build(CHECK_PARENT_TASK_NAME)
        }
    }

    @DisplayName("Format task should be UP-TO-DATE on 3rd run")
    @Test
    fun formatUpToDate() {
        project {
            withFailingSources()
            val formatTaskName = KtLintFormatTask.buildTaskNameForSourceSet("main")

            build(FORMAT_PARENT_TASK_NAME) {
                assertThat(task(":$formatTaskName")?.outcome).isEqualTo(TaskOutcome.SUCCESS)
            }
            build(FORMAT_PARENT_TASK_NAME) {
                assertThat(task(":$formatTaskName")?.outcome).isEqualTo(TaskOutcome.SUCCESS)
            }
            build(FORMAT_PARENT_TASK_NAME) {
                assertThat(task(":$formatTaskName")?.outcome).isEqualTo(TaskOutcome.UP_TO_DATE)
            }
        }
    }

    @DisplayName("Format task should not create directories for empty SourceSets")
    @Test
    fun formatNotCreateEmpty() {
        project {
            withFailingSources()

            build(FORMAT_PARENT_TASK_NAME) {
                assertThat(projectPath.resolve("src/main/java").exists()).isFalse()
            }
        }
    }

    @DisplayName("Format task should succeed on renamed file")
    @Test
    fun formatShouldSucceedOnRenamedFile() {
        project {
            withFailingSources()

            val formatTaskName = KtLintFormatTask.buildTaskNameForSourceSet("main")
            build(FORMAT_PARENT_TASK_NAME) {
                assertThat(task(":$formatTaskName")?.outcome).isEqualTo(TaskOutcome.SUCCESS)
            }

            val sourceFile = projectPath.resolve(FAIL_SOURCE_FILE)
            sourceFile.writeText(
                """
                val  foo    =    "bar"
            """
            )
            sourceFile.renameTo(projectPath.resolve("src/main/kotlin/RenamedFile.kt"))

            build(FORMAT_PARENT_TASK_NAME) {
                assertThat(task(":$formatTaskName")?.outcome).isEqualTo(TaskOutcome.SUCCESS)
            }

            build(CHECK_PARENT_TASK_NAME)
        }
    }

    @DisplayName("Should print paths to the generated reports on code style violations")
    @Test
    fun printReportsPaths() {
        project {
            withFailingSources()

            buildAndFail(CHECK_PARENT_TASK_NAME) {
                val s = File.separator
                assertThat(output)
                    .contains("build${s}reports${s}ktlint${s}ktlintMainSourceSetCheck${s}ktlintMainSourceSetCheck.txt")
            }
        }
    }

    @DisplayName("Should not leak KtLint into buildscript classpath")
    @Test
    fun noLeakIntoBuildscriptClasspath() {
        project {
            withCleanSources()

            build("buildEnvironment") { assertThat(output).doesNotContain("com.pinterest.ktlint") }
        }
    }

    @DisplayName("Should not leak KtLint as a variant into consuming projects")
    @Test
    fun noLeakIntoConsumingProjects() {
        project {
            val producerDir = projectPath.resolve("producer").also { it.mkdirs() }
            val consumerDir = projectPath.resolve("consumer").also { it.mkdirs() }
            settingsGradle.appendText("\ninclude \":producer\"\ninclude \":consumer\"\n")
            producerDir.buildFile().writeText(
                """
                plugins {
                    id "$PLUGIN_ID"
                }
                configurations.create('default')
                artifacts {
                    add('default', buildFile)
                }
                """
                    .trimIndent()
            )
            consumerDir.buildFile().writeText(
                """
                plugins {
                    id 'java'
                }
                dependencies {
                    implementation project(':producer')
                }
                """
                    .trimIndent()
            )

            build(":consumer:dependencies") { assertThat(output).doesNotContain("com.pinterest:ktlint") }
        }
    }

    @DisplayName("Should add check on additional sources")
    @Test
    fun checkAdditionalSources() {
        fun setup(file: File) {
            projectSetup("jvm").invoke(file)
            file.resolve("build.gradle").appendText(
                """
                |
                |sourceSets {
                |  additionalSources {
                |    kotlin {
                |      srcDir 'src/additionalSources/kotlin'
                |    }
                |  }
                |}
                """
                    .trimMargin()
            )
        }

        val additionalCheck =
            GenerateReportsTask.generateNameForSourceSets("additionalSources", GenerateReportsTask.LintType.CHECK)

        project(projectSetup = ::setup) {
            build("-m", CHECK_PARENT_TASK_NAME) {
                val lines = output.lineSequence().toList()
                assertThat(lines.any { it.contains(mainSourceSetCheckTaskName) }).isTrue()
                assertThat(lines.any { it.contains(additionalCheck) }).isTrue()
            }
        }
    }
}
