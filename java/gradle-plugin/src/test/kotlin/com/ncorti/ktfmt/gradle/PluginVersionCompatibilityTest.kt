package com.ncorti.ktfmt.gradle

import com.google.common.truth.Truth.assertThat
import com.ncorti.ktfmt.gradle.testutil.copyFixture
import com.ncorti.ktfmt.gradle.testutil.createTempFile
import com.ncorti.ktfmt.gradle.testutil.gradle
import java.io.File
import org.gradle.testkit.runner.TaskOutcome
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.io.TempDir
import org.junit.jupiter.params.ParameterizedTest
import org.junit.jupiter.params.provider.ValueSource

internal class PluginVersionCompatibilityTest {

    @TempDir lateinit var tempDir: File

    @BeforeEach
    fun setUp() {
        tempDir.copyFixture("jvmProject-version-compatibility")
    }

    @ParameterizedTest
    @ValueSource(strings = ["1.7.20", "1.9.10", "1.9.20", "2.0.0", "2.4.10"])
    fun `plugin can be applied to projects with different kotlin versions`(kotlinVersion: String) {
        replaceKotlinVersion(kotlinVersion)

        tempDir.createTempFile(content = "val answer = 42\n")

        val result = tempDir.gradle("ktfmtCheckMain", "--info").build()

        assertThat(result.task(":ktfmtCheckMain")?.outcome).isEqualTo(TaskOutcome.SUCCESS)
    }

    private fun replaceKotlinVersion(version: String) {
        val file = tempDir.resolve("build.gradle.kts")
        val updatedKotlinVersion = file.readText().replace("KOTLIN_VERSION_PLACEHOLDER", version)

        file.writeText(updatedKotlinVersion)
    }
}
