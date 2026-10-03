package org.jlleitschuh.gradle.ktlint

import com.google.common.truth.Truth.assertThat
import org.gradle.testkit.runner.TaskOutcome
import org.jlleitschuh.gradle.ktlint.reporter.ReporterType
import org.jlleitschuh.gradle.ktlint.testdsl.TestProject
import org.jlleitschuh.gradle.ktlint.testdsl.build
import org.jlleitschuh.gradle.ktlint.testdsl.buildAndFail
import org.jlleitschuh.gradle.ktlint.testdsl.project
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.Test

/** Upstream's ReportersTest, without the 3rd party reporter case (it downloads the reporter). */
class ReportersTest : AbstractPluginTest() {

    @DisplayName("Should create multiple reports")
    @Test
    fun multipleReports() {
        project {
            buildGradle.appendText("\nktlint.reporters {\n    reporter \"checkstyle\"\n    reporter \"json\"\n}\n")
            withFailingSources()

            buildAndFail(CHECK_PARENT_TASK_NAME) {
                assertThat(task(":$mainSourceSetCheckTaskName")?.outcome).isEqualTo(TaskOutcome.FAILED)
                assertThat(output).contains("Unnecessary long whitespace")
                assertReportNotCreated(ReporterType.PLAIN.fileExtension, mainSourceSetCheckTaskName)
                assertReportCreated(ReporterType.CHECKSTYLE.fileExtension, mainSourceSetCheckTaskName)
                assertReportCreated(ReporterType.JSON.fileExtension, mainSourceSetCheckTaskName)
            }
        }
    }

    @DisplayName("Task is not UP-TO-DATE when another reporter was enabled in the build script")
    @Test
    fun anotherReporterNotUpToDate() {
        project {
            withCleanSources()
            buildGradle.appendText("\nktlint.reporters {\n    reporter \"json\"\n    reporter \"plain\"\n}\n")

            build(CHECK_PARENT_TASK_NAME) {
                assertThat(task(":$mainSourceSetCheckTaskName")?.outcome).isEqualTo(TaskOutcome.SUCCESS)
                assertReportCreated(ReporterType.PLAIN.fileExtension, mainSourceSetCheckTaskName)
                assertReportCreated(ReporterType.JSON.fileExtension, mainSourceSetCheckTaskName)
            }

            build(CHECK_PARENT_TASK_NAME) {
                assertThat(task(":$mainSourceSetCheckTaskName")?.outcome).isEqualTo(TaskOutcome.UP_TO_DATE)
                assertReportCreated(ReporterType.PLAIN.fileExtension, mainSourceSetCheckTaskName)
                assertReportCreated(ReporterType.JSON.fileExtension, mainSourceSetCheckTaskName)
                assertReportNotCreated(ReporterType.CHECKSTYLE.fileExtension, mainSourceSetCheckTaskName)
            }

            buildGradle.appendText(
                "\nktlint.reporters {\n    reporter \"json\"\n    reporter \"plain_group_by_file\"\n}\n"
            )

            build(CHECK_PARENT_TASK_NAME) {
                assertThat(task(":$mainSourceSetCheckTaskName")?.outcome).isEqualTo(TaskOutcome.SUCCESS)
                assertReportCreated(ReporterType.PLAIN_GROUP_BY_FILE.fileExtension, mainSourceSetCheckTaskName)
                assertReportCreated(ReporterType.JSON.fileExtension, mainSourceSetCheckTaskName)
                assertReportNotCreated(ReporterType.CHECKSTYLE.fileExtension, mainSourceSetCheckTaskName)
            }

            buildGradle.appendText("\nktlint.reporters {\n    reporter \"json\"\n    reporter \"checkstyle\"\n}\n")

            build(CHECK_PARENT_TASK_NAME) {
                assertThat(task(":$mainSourceSetCheckTaskName")?.outcome).isEqualTo(TaskOutcome.SUCCESS)
                assertReportCreated(ReporterType.JSON.fileExtension, mainSourceSetCheckTaskName)
                assertReportCreated(ReporterType.CHECKSTYLE.fileExtension, mainSourceSetCheckTaskName)
                assertReportCreated(ReporterType.PLAIN.fileExtension, mainSourceSetCheckTaskName)
            }
        }
    }

    @DisplayName("Should use plain reporter if no reporters are defined")
    @Test
    fun defaultReporter() {
        project {
            withFailingSources()

            buildAndFail(CHECK_PARENT_TASK_NAME) {
                assertThat(task(":$mainSourceSetCheckTaskName")?.outcome).isEqualTo(TaskOutcome.FAILED)
                assertReportCreated(ReporterType.PLAIN.fileExtension, mainSourceSetCheckTaskName)
                assertReportNotCreated(ReporterType.CHECKSTYLE.fileExtension, mainSourceSetCheckTaskName)
                assertReportNotCreated(ReporterType.JSON.fileExtension, mainSourceSetCheckTaskName)
            }
        }
    }

    @DisplayName("Should generate html report")
    @Test
    fun htmlReport() {
        assertSingleReporterReport(ReporterType.HTML, "html")
    }

    @DisplayName("Should generate sarif report")
    @Test
    fun sarifReport() {
        assertSingleReporterReport(ReporterType.SARIF, "sarif")
    }

    private fun assertSingleReporterReport(type: ReporterType, name: String) {
        project {
            withCleanSources()
            buildGradle.appendText("\nktlint.reporters {\n    reporter \"$name\"\n}\n")

            build(CHECK_PARENT_TASK_NAME) {
                assertThat(task(":$mainSourceSetCheckTaskName")?.outcome).isEqualTo(TaskOutcome.SUCCESS)
                assertReportCreated(type.fileExtension, mainSourceSetCheckTaskName)
            }
        }
    }

    @DisplayName("Should allow to set custom location for generated reports")
    @Test
    fun customReportsLocation() {
        project {
            withFailingSources()

            val newLocation = "other/location"
            buildGradle.appendText(
                """

                ktlint.reporters {
                    reporter "checkstyle"
                    reporter "json"
                }

                tasks.withType(org.jlleitschuh.gradle.ktlint.tasks.GenerateReportsTask.class) {
                    reportsOutputDirectory.set(project.layout.buildDirectory.dir("$newLocation/${'$'}name"))
                }
                """
                    .trimIndent()
            )

            buildAndFail(CHECK_PARENT_TASK_NAME) {
                assertThat(task(":$mainSourceSetCheckTaskName")?.outcome).isEqualTo(TaskOutcome.FAILED)
                val location = "build/$newLocation/$mainSourceSetCheckTaskName"
                assertReportNotCreated(ReporterType.CHECKSTYLE.fileExtension, mainSourceSetCheckTaskName)
                assertReportCreated(ReporterType.CHECKSTYLE.fileExtension, mainSourceSetCheckTaskName, location)
                assertReportNotCreated(ReporterType.JSON.fileExtension, mainSourceSetCheckTaskName)
                assertReportCreated(ReporterType.JSON.fileExtension, mainSourceSetCheckTaskName, location)
            }
        }
    }

    private fun TestProject.assertReportCreated(
        extension: String,
        taskName: String,
        baseLocation: String = "build/reports/ktlint/$taskName",
    ) {
        assertThat(projectPath.resolve("$baseLocation/$taskName.$extension").isFile).isTrue()
    }

    private fun TestProject.assertReportNotCreated(
        extension: String,
        taskName: String,
        baseLocation: String = "build/reports/ktlint/$taskName",
    ) {
        assertThat(projectPath.resolve("$baseLocation/$taskName.$extension").isFile).isFalse()
    }
}
