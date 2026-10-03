package org.jlleitschuh.gradle.ktlint

import org.jlleitschuh.gradle.ktlint.testdsl.build
import org.jlleitschuh.gradle.ktlint.testdsl.project
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.params.ParameterizedTest
import org.junit.jupiter.params.provider.ValueSource

class TaskConfigurationAvoidanceTest : AbstractPluginTest() {

    @DisplayName("should support configuration avoidance")
    @ParameterizedTest(name = "task {0} {displayName}")
    @ValueSource(strings = ["LoadReportersTask", "GenerateReportsTask", "KtLintCheckTask", "KtLintFormatTask"])
    fun checkTaskAvoidance(taskName: String) {
        project {
            buildGradle.appendText(
                """

                tasks
                     .withType(org.jlleitschuh.gradle.ktlint.tasks.$taskName.class)
                     .configureEach {
                          throw new RuntimeException("Created on configuration phase")
                     }
                """
                    .trimIndent()
            )

            build("help", "-s")
        }
    }
}
