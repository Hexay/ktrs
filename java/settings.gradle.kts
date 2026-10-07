pluginManagement {
    repositories {
        mavenCentral()
        gradlePluginPortal()
    }
}

rootProject.name = "ktrs"

include(":ktrs-gradle-plugin")
project(":ktrs-gradle-plugin").projectDir = file("gradle-plugin")

include(":ktrs-ktlint-maven-plugin")
project(":ktrs-ktlint-maven-plugin").projectDir = file("maven-plugin")
