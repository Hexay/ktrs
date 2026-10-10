package org.jmailen.gradle.kotlinter.functional

import java.io.File
import org.gradle.testkit.runner.TaskOutcome
import org.jmailen.gradle.kotlinter.functional.utils.PLUGIN_ID
import org.jmailen.gradle.kotlinter.functional.utils.kotlinClass
import org.jmailen.gradle.kotlinter.functional.utils.resolve
import org.jmailen.gradle.kotlinter.functional.utils.settingsFile
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertFalse
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test

class WorkerJvmArgsTest : WithGradleTest.Kotlin() {

    lateinit var projectRoot: File

    @BeforeEach
    fun setUp() {
        projectRoot =
            testProjectDir.apply {
                resolve("settings.gradle") { writeText(settingsFile) }
                resolve("build.gradle") {
                    val buildScript =
                        """
                        plugins {
                            id 'kotlin'
                            id '$PLUGIN_ID'
                        }

                        repositories {
                            mavenCentral()
                        }
                        """.trimIndent()
                    writeText(buildScript)
                }
                resolve("src/main/kotlin/CustomClass.kt") { writeText(kotlinClass("CustomClass")) }
            }
    }

    @Test
    fun `lint task doesn't print the sun misc Unsafe deprecation warning`() {
        build("lintKotlin").apply {
            assertEquals(TaskOutcome.SUCCESS, task(":lintKotlinMain")?.outcome)
            assertFalse(output.contains("sun.misc.Unsafe"), "unexpected JDK deprecation warning in build output")
        }
    }

    /** Upstream's `workerJvmArgs are passed to the worker jvm` has the worker JVM reject the option; ktrs runs none. */
    @Test
    fun `workerJvmArgs are accepted and the lint task succeeds without a worker jvm`() {
        projectRoot.resolve("build.gradle") {
            val buildScript =
                """

                import org.jmailen.gradle.kotlinter.tasks.LintTask

                tasks.withType(LintTask).configureEach {
                    workerJvmArgs.add('--not-a-jvm-option')
                }
                """.trimIndent()
            appendText(buildScript)
        }

        build("lintKotlin").apply {
            assertEquals(TaskOutcome.SUCCESS, task(":lintKotlinMain")?.outcome)
            assertFalse(output.contains("Unrecognized option: --not-a-jvm-option"))
        }
    }
}
