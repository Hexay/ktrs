plugins {
    kotlin("jvm") version "2.4.20"
}

kotlin {
    jvmToolchain(21)
}

dependencies {
    implementation("com.pinterest.ktlint:ktlint-rule-engine:1.8.0")
}
