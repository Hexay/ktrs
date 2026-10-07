plugins {
    `kotlin-dsl`
}

repositories {
    gradlePluginPortal()
}

dependencies {
    implementation(libs.ktfmt.gradle.plugin)
    implementation(libs.kotlin.gradle.plugin)
}
