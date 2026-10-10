package org.jmailen.gradle.kotlinter.functional.utils

import java.io.File
import org.junit.jupiter.api.Assumptions

internal fun File.resolve(path: String, receiver: File.() -> Unit): File =
    resolve(path).apply {
        parentFile.mkdirs()
        receiver()
    }

/** The ktrs binary under test as the plugin's Gradle property; an argument, as tests write `gradle.properties`. */
internal val ktrsExecutableArgument: String
    get() = "-Pktrs.executable=" + System.getProperty("ktrs.executable").replace('\\', '/')

/** Skips the test where the Android Gradle plugin would find no SDK. */
internal fun assumeAndroidSdk() {
    val sdk = System.getenv("ANDROID_HOME") ?: System.getenv("ANDROID_SDK_ROOT")
    Assumptions.assumeTrue(!sdk.isNullOrEmpty(), "ANDROID_HOME or ANDROID_SDK_ROOT is not set")
}
