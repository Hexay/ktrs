plugins {
    kotlin("jvm") version "2.4.20"
    id("io.github.hexay.ktrs.kotlinter") version "0.5.0"
}

kotlinter {
    ktlintVersion = "2.0.0-ALPHA-4"
}

dependencies {
    ktlint("io.nlopez.compose.rules:ktlint:0.6.7")
}
