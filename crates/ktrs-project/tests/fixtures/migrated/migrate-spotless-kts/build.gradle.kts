import com.diffplug.gradle.spotless.SpotlessExtension

buildscript {
    repositories { mavenCentral() }
    dependencies { classpath("io.github.hexay:ktrs:0.5.0") }
}

plugins {
    id("com.diffplug.spotless") version "8.0.0"
}

spotless {
    kotlin {
        addStep(io.github.hexay.ktrs.spotless.KtrsStep.create(io.github.hexay.ktrs.KtrsOptions.kotlinlang()))
        addStep(io.github.hexay.ktrs.spotless.KtrsKtlintStep.create(io.github.hexay.ktrs.KtlintOptions.defaults()
            .withEditorConfigPath(rootProject.file(".editorconfig"))
            .withEditorConfigOverride(mapOf("ktlint_code_style" to "ktlint_official"))))
    }
}
