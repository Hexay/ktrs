import com.diffplug.gradle.spotless.SpotlessExtension

plugins {
    id("com.diffplug.spotless") version "8.0.0"
}

spotless {
    kotlin {
        ktfmt().kotlinlangStyle()
        ktlint("1.8.0")
            .editorConfigOverride(mapOf("ktlint_code_style" to "ktlint_official"))
    }
}
