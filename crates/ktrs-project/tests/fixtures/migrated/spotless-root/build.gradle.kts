buildscript {
    repositories { mavenCentral() }
    dependencies { classpath("io.github.hexay:ktrs:0.5.0") }
}

plugins {
    id("com.diffplug.spotless") version "8.0.0"
}

val ktlintVersion = "1.8.0"

spotless {
    kotlin {
        target("**/*.kt")
        addStep(io.github.hexay.ktrs.spotless.KtrsStep.create(io.github.hexay.ktrs.KtrsOptions.google().withMaxWidth(80).withBlockIndent(2)))
    }
    kotlinGradle {
        target("*.gradle.kts")
        ktlint(ktlintVersion)
            .editorConfigOverride(mapOf("indent_size" to 2))
            .customRuleSets(listOf("io.nlopez.compose.rules:ktlint:0.4.22"))
    }
}
