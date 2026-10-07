plugins {
    `kotlin-dsl`
}

repositories {
    gradlePluginPortal()
    maven("https://hexay.github.io/ktrs/maven")
}

dependencies {
    implementation(libs.ktfmt.gradle.plugin)
    implementation(libs.kotlin.gradle.plugin)
}
