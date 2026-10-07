plugins {
    `kotlin-dsl`
}

repositories {
    gradlePluginPortal()
    maven("https://hexay.github.io/ktrs/maven")
}

dependencies {
    implementation("io.github.hexay:ktrs-gradle-plugin:0.5.0")
}
