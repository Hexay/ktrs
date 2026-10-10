plugins {
    id("com.diffplug.spotless") version "8.0.0"
}

val ktlintVersion = "1.8.0"

spotless {
    kotlin {
        target("**/*.kt")
        ktfmt("0.65").googleStyle().configure {
            it.setMaxWidth(80)
            it.setBlockIndent(2)
        }
    }
    kotlinGradle {
        target("*.gradle.kts")
        ktlint(ktlintVersion)
            .editorConfigOverride(mapOf("indent_size" to 2))
            .customRuleSets(listOf("io.nlopez.compose.rules:ktlint:0.4.22"))
    }
}
