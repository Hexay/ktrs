plugins {
    kotlin("jvm") version "2.4.20"
    id("org.jmailen.kotlinter") version "4.4.1"
}

kotlinter {
    ktlintVersion = "1.5.0"
    ignoreFailures = true
}

dependencies {
    ktlint("some.group:custom-rules:1.0")
    ktlint("io.nlopez.compose.rules:ktlint:0.6.7")
    ktlint(project(":rules"))
}
