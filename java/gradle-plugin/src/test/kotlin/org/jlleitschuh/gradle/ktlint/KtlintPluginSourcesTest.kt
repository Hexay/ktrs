package org.jlleitschuh.gradle.ktlint

import com.google.common.truth.Truth.assertThat
import org.gradle.testkit.runner.TaskOutcome
import org.jlleitschuh.gradle.ktlint.testdsl.build
import org.jlleitschuh.gradle.ktlint.testdsl.buildAndFail
import org.jlleitschuh.gradle.ktlint.testdsl.project
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.condition.EnabledOnOs
import org.junit.jupiter.api.condition.OS

/** Upstream KtlintPluginTest's source selection cases: filters, scripts, git filter, changed files. */
class KtlintPluginSourcesTest : AbstractPluginTest() {

    @DisplayName("Should ignore excluded sources")
    @Test
    fun ignoreExcludedSources() {
        project {
            withCleanSources()
            withFailingSources()
            buildGradle.appendText("\nktlint.filter { exclude(\"**/FailSource.kt\") }\n")

            build(CHECK_PARENT_TASK_NAME) {
                assertThat(task(":$mainSourceSetCheckTaskName")?.outcome).isEqualTo(TaskOutcome.SUCCESS)
            }
        }
    }

    @DisplayName("Should fail on additional source set directories files style violation")
    @Test
    fun additionalSourceSetsViolations() {
        project {
            withCleanSources()
            val alternativeDirectory = "src/main/shared"
            projectPath.withAlternativeFailingSources(alternativeDirectory)
            buildGradle.appendText(
                "\nsourceSets {\n    findByName(\"main\")?.java?.srcDirs(project.file(\"$alternativeDirectory\"))\n}\n"
            )

            buildAndFail(CHECK_PARENT_TASK_NAME) {
                assertThat(task(":$mainSourceSetCheckTaskName")?.outcome).isEqualTo(TaskOutcome.FAILED)
            }
        }
    }

    @DisplayName("Should check Kotlin script file in project folder")
    @Test
    fun checkKotlinScript() {
        project {
            withCleanSources()
            withCleanKotlinScript()

            build(kotlinScriptCheckTaskName) {
                assertThat(task(":$kotlinScriptCheckTaskName")?.outcome).isEqualTo(TaskOutcome.SUCCESS)
            }
        }
    }

    @DisplayName("Should fail check for Kotlin script file in project folder with style violations")
    @Test
    fun checkAndFailKotlinScript() {
        project {
            withCleanSources()
            withFailingKotlinScript()

            buildAndFail(kotlinScriptCheckTaskName) {
                assertThat(task(":$kotlinScriptCheckTaskName")?.outcome).isEqualTo(TaskOutcome.FAILED)
            }
        }
    }

    @DisplayName("Should not check Kotlin script file in child project folder")
    @Test
    fun ignoreKotlinScript() {
        project {
            withCleanSources()
            projectPath.resolve("scripts/").also { it.mkdirs() }.withFailingKotlinScript()

            build(kotlinScriptCheckTaskName) {
                assertThat(task(":$kotlinScriptCheckTaskName")?.outcome).isEqualTo(TaskOutcome.SKIPPED)
            }
        }
    }

    @DisplayName("Should check kts files in configured child project folder")
    @Test
    fun checkAdditionallyAddedKtsFiles() {
        project {
            withCleanSources()
            projectPath.resolve("scripts/").withCleanKotlinScript()
            buildGradle.appendText("\nktlint.kotlinScriptAdditionalPaths { include fileTree(\"scripts/\") }\n")

            build(kotlinScriptCheckTaskName) {
                assertThat(task(":$kotlinScriptCheckTaskName")?.outcome).isEqualTo(TaskOutcome.SUCCESS)
            }
        }
    }

    @DisplayName("Should apply internal git filter to check task")
    @Test
    fun gitFilterOnCheck() {
        assertGitFilterSelectsCleanSource("src/main/kotlin/CleanSource.kt")
    }

    @DisplayName("Internal Git filter works with Windows")
    @Test
    @EnabledOnOs(OS.WINDOWS)
    fun gitFilterOnCheckWindows() {
        assertGitFilterSelectsCleanSource("src\\main\\kotlin\\CleanSource.kt")
    }

    private fun assertGitFilterSelectsCleanSource(filter: String) {
        project {
            withCleanSources()
            withFailingSources()

            build(":$CHECK_PARENT_TASK_NAME", "-P$FILTER_INCLUDE_PROPERTY_NAME=$filter") {
                assertThat(task(":$mainSourceSetCheckTaskName")?.outcome).isEqualTo(TaskOutcome.SUCCESS)
            }
        }
    }

    @DisplayName("Git filter should respect already applied filters")
    @Test
    fun gitFilterAlreadyAppliedFilters() {
        project {
            withFailingSources()
            buildGradle.appendText("\nktlint.filter { exclude(\"**/FailSource.kt\") }\n")

            build(":$CHECK_PARENT_TASK_NAME", "-P$FILTER_INCLUDE_PROPERTY_NAME=src/main/kotlin/FailSource.kt") {
                assertThat(task(":$mainSourceSetCheckTaskName")?.outcome).isEqualTo(TaskOutcome.SKIPPED)
            }
        }
    }

    @DisplayName("Git filter should ignore task if no files related to it")
    @Test
    fun gitFilterIgnoreTask() {
        project {
            withCleanSources()

            build(":$CHECK_PARENT_TASK_NAME", "-P$FILTER_INCLUDE_PROPERTY_NAME=src/main/kotlin/failing-sources.kt") {
                assertThat(task(":$mainSourceSetCheckTaskName")?.outcome).isEqualTo(TaskOutcome.SKIPPED)
            }
        }
    }

    @DisplayName("Lint check should run again on an added file")
    @Test
    fun checkRerunsOnAddedFile() {
        project {
            createSourceFile("src/main/kotlin/Initial.kt", "val foo = \"bar\"\n")

            build(CHECK_PARENT_TASK_NAME) {
                assertThat(task(":$mainSourceSetCheckTaskName")?.outcome).isEqualTo(TaskOutcome.SUCCESS)
            }

            createSourceFile("src/main/kotlin/AnotherFile.kt", "val bar = \"foo\"\n")

            build(CHECK_PARENT_TASK_NAME) {
                assertThat(task(":runKtlintCheckOverMainSourceSet")?.outcome).isEqualTo(TaskOutcome.SUCCESS)
            }
        }
    }

    @DisplayName("Lint check should repeat errors of unchanged files")
    @Test
    fun checkRepeatsErrors() {
        project {
            createSourceFile("src/main/kotlin/Initial.kt", "val foo=\"bar\"\n")

            buildAndFail(CHECK_PARENT_TASK_NAME) {
                assertThat(task(":$mainSourceSetCheckTaskName")?.outcome).isEqualTo(TaskOutcome.FAILED)
            }

            createSourceFile("src/main/kotlin/AnotherFile.kt", "val bar=\"foo\"\n")

            buildAndFail(CHECK_PARENT_TASK_NAME) {
                assertThat(task(":$mainSourceSetCheckTaskName")?.outcome).isEqualTo(TaskOutcome.FAILED)
                assertThat(output).contains("Initial.kt")
                assertThat(output).contains("AnotherFile.kt")
            }
        }
    }

    @DisplayName("Should check files which path contains whitespace")
    @Test
    fun pathsWithWhitespace() {
        project {
            createSourceFile("src/main/kotlin/some path with whitespace/some file.kt", "class Test")

            buildAndFail(CHECK_PARENT_TASK_NAME) {
                assertThat(task(":$mainSourceSetCheckTaskName")?.outcome).isEqualTo(TaskOutcome.FAILED)
            }
        }
    }

    @DisplayName("Should do nothing when there are no eligible incremental updates")
    @Test
    fun noIncrementalUpdates() {
        project {
            val passing = "val foo = \"bar\"\n"
            createSourceFile("src/main/kotlin/Initial.kt", passing)
            createSourceFile("src/main/kotlin/AnotherFile.kt", passing)
            createSourceFile("src/test/kotlin/AnotherFile.kt", "val foo=\"bar\"\n")

            build(mainSourceSetCheckTaskName) {
                assertThat(task(":$mainSourceSetCheckTaskName")?.outcome).isEqualTo(TaskOutcome.SUCCESS)
            }

            removeSourceFile("src/main/kotlin/Initial.kt")
            build(mainSourceSetCheckTaskName) {
                assertThat(task(":$mainSourceSetCheckTaskName")?.outcome).isEqualTo(TaskOutcome.UP_TO_DATE)
            }
        }
    }

    @DisplayName("Lint check should pass after file is deleted")
    @Test
    fun checkAfterFileDelete() {
        project {
            createSourceFile("src/main/kotlin/FileOne.kt", "val foo = \"bar\"\n")
            createSourceFile("src/main/kotlin/FileTwo.kt", "val bar = \"foo\"\n")

            build(CHECK_PARENT_TASK_NAME) {
                assertThat(task(":$mainSourceSetCheckTaskName")?.outcome).isEqualTo(TaskOutcome.SUCCESS)
            }

            removeSourceFile("src/main/kotlin/FileOne.kt")
            createSourceFile("src/main/kotlin/FileThree.kt", "val bar = \"foo\"\n")

            build(CHECK_PARENT_TASK_NAME) {
                assertThat(task(":$mainSourceSetCheckTaskName")?.outcome).isEqualTo(TaskOutcome.SUCCESS)
            }
        }
    }

    @DisplayName("Lint check should run incrementally")
    @Test
    fun checkIsIncremental() {
        project {
            createSourceFile("src/main/kotlin/Initial.kt", "val foo = \"bar\"\n")
            build(CHECK_PARENT_TASK_NAME) {
                assertThat(task(":$mainSourceSetCheckTaskName")?.outcome).isEqualTo(TaskOutcome.SUCCESS)
            }

            createSourceFile("src/main/kotlin/AnotherFile.kt", "val bar = \"foo\"\n")
            build(CHECK_PARENT_TASK_NAME, "--info") {
                assertThat(task(":$mainSourceSetCheckTaskName")?.outcome).isEqualTo(TaskOutcome.SUCCESS)
                assertThat(output).contains("Executing incrementally")
            }
        }
    }

    @DisplayName("Lint check should run incrementally and repeat errors")
    @Test
    fun checkIsIncrementalWithErrors() {
        project {
            createSourceFile("src/main/kotlin/Initial.kt", "val foo=\"bar\"\n")
            buildAndFail(CHECK_PARENT_TASK_NAME) {
                assertThat(task(":$mainSourceSetCheckTaskName")?.outcome).isEqualTo(TaskOutcome.FAILED)
            }

            createSourceFile("src/main/kotlin/AnotherFile.kt", "val bar=\"foo\"\n")
            buildAndFail(CHECK_PARENT_TASK_NAME, "--info") {
                assertThat(task(":$mainSourceSetCheckTaskName")?.outcome).isEqualTo(TaskOutcome.FAILED)
                assertThat(output).contains("Executing incrementally")
                assertThat(output).contains("Initial.kt")
                assertThat(output).contains("AnotherFile.kt")
            }
        }
    }
}
