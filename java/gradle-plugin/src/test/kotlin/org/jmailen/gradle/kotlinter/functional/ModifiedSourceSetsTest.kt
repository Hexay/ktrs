package org.jmailen.gradle.kotlinter.functional

import java.io.File
import org.gradle.testkit.runner.TaskOutcome
import org.jmailen.gradle.kotlinter.functional.utils.PLUGIN_ID
import org.jmailen.gradle.kotlinter.functional.utils.androidManifest
import org.jmailen.gradle.kotlinter.functional.utils.androidRootBuildScript
import org.jmailen.gradle.kotlinter.functional.utils.assumeAndroidSdk
import org.jmailen.gradle.kotlinter.functional.utils.kotlinClass
import org.jmailen.gradle.kotlinter.functional.utils.resolve
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test

internal class ModifiedSourceSetsTest : WithGradleTest.Android() {

    private lateinit var androidModuleRoot: File
    private lateinit var kotlinModuleRoot: File

    @BeforeEach
    fun setUp() {
        assumeAndroidSdk()
        testProjectDir.apply {
            resolve("settings.gradle") { writeText(settingsFile) }
            resolve("build.gradle") { writeText(androidRootBuildScript) }
            resolve("gradle.properties") {
                writeText(
                    """
                    android.builtInKotlin=false
                    android.newDsl=false
                    """.trimIndent()
                )
            }
            androidModuleRoot =
                resolve("androidproject") {
                    resolve("build.gradle") {
                        val androidBuildScript =
                            """
                            plugins {
                                id 'com.android.library'
                                id 'kotlin-android'
                                id '$PLUGIN_ID'
                            }

                            android {
                                namespace = "com.example"

                                compileSdkVersion 31
                                defaultConfig {
                                    minSdkVersion 23
                                }
                                sourceSets {
                                    main.java.srcDirs += "src/main/kotlin"
                                    test.java.srcDirs += "src/test/kotlin"
                                    debug.java.srcDirs += "src/debug/kotlin"
                                    flavorOne.java.srcDirs = ['src/customFolder/kotlin']
                                }

                                flavorDimensions 'customFlavor'
                                productFlavors {
                                    flavorOne {
                                        dimension 'customFlavor'
                                    }
                                    flavorTwo {
                                        dimension 'customFlavor'
                                    }
                                }
                            }

                            """.trimIndent()
                        writeText(androidBuildScript)
                    }
                    resolve("src/main/AndroidManifest.xml") { writeText(androidManifest) }
                    resolve("src/main/kotlin/com/example/MainSourceSet.kt") { writeText(kotlinClass("MainSourceSet")) }
                    resolve("src/debug/kotlin/com/example/DebugSourceSet.kt") { writeText(kotlinClass("DebugSourceSet")) }
                    resolve("src/test/kotlin/com/example/TestSourceSet.kt") { writeText(kotlinClass("TestSourceSet")) }
                    resolve("src/customFolder/kotlin/com/example/CustomSourceSet.kt") {
                        writeText(kotlinClass("CustomSourceSet"))
                    }
                }
            kotlinModuleRoot =
                resolve("kotlinproject") {
                    resolve("build.gradle") {
                        val kotlinBuildScript =
                            """
                            plugins {
                                id 'kotlin'
                                id '$PLUGIN_ID'
                            }

                            sourceSets {
                                main.kotlin.srcDirs += "random/path"
                                individuallyCustomized {
                                    java.srcDirs = ["src/debug/kotlin"]
                                }
                            }
                            """.trimIndent()
                        writeText(kotlinBuildScript)
                    }
                    resolve("random/path/MainSourceSet.kt") { writeText(kotlinClass("MainSourceSet")) }
                    resolve("src/test/kotlin/TestSourceSet.kt") { writeText(kotlinClass("TestSourceSet")) }
                    resolve("src/individuallyCustomized/kotlin/CustomSourceSet.kt") {
                        writeText(kotlinClass("CustomSourceSet"))
                    }
                }
        }
    }

    @Test
    fun `kotlinter detects sources in all sourcesets`() {
        build("lintKotlin").apply {
            assertEquals(TaskOutcome.SUCCESS, task(":androidproject:lintKotlinMain")?.outcome)
            assertEquals(TaskOutcome.SUCCESS, task(":androidproject:lintKotlinDebug")?.outcome)
            assertEquals(TaskOutcome.SUCCESS, task(":androidproject:lintKotlinTest")?.outcome)
            assertEquals(TaskOutcome.SUCCESS, task(":androidproject:lintKotlinFlavorOne")?.outcome)
            assertEquals(TaskOutcome.SUCCESS, task(":androidproject:lintKotlin")?.outcome)
            assertEquals(TaskOutcome.SUCCESS, task(":kotlinproject:lintKotlinMain")?.outcome)
            assertEquals(TaskOutcome.SUCCESS, task(":kotlinproject:lintKotlinTest")?.outcome)
            assertEquals(TaskOutcome.SUCCESS, task(":kotlinproject:lintKotlinIndividuallyCustomized")?.outcome)
        }
    }

    @Test
    fun `kotlinter becomes up-to-date on second run`() {
        build("lintKotlin")

        build("lintKotlin").apply {
            assertEquals(TaskOutcome.UP_TO_DATE, task(":androidproject:lintKotlinMain")?.outcome)
            assertEquals(TaskOutcome.UP_TO_DATE, task(":androidproject:lintKotlinDebug")?.outcome)
            assertEquals(TaskOutcome.UP_TO_DATE, task(":androidproject:lintKotlinTest")?.outcome)
            assertEquals(TaskOutcome.UP_TO_DATE, task(":androidproject:lintKotlinFlavorOne")?.outcome)
            assertEquals(TaskOutcome.UP_TO_DATE, task(":androidproject:lintKotlin")?.outcome)
            assertEquals(TaskOutcome.UP_TO_DATE, task(":kotlinproject:lintKotlinMain")?.outcome)
            assertEquals(TaskOutcome.UP_TO_DATE, task(":kotlinproject:lintKotlinTest")?.outcome)
            assertEquals(TaskOutcome.UP_TO_DATE, task(":kotlinproject:lintKotlinIndividuallyCustomized")?.outcome)
        }
    }

    private val settingsFile =
        """
        rootProject.name = 'kotlinter'
        include 'androidproject', 'kotlinproject'
        """.trimIndent()
}
