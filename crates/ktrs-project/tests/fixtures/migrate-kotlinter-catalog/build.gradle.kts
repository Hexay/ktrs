plugins {
    alias(libs.plugins.kotlin.jvm)
    alias(libs.plugins.kotlinter)
}

kotlinter {
    ktlintVersion = "1.8.0"
    reporters = arrayOf("checkstyle", "plain")
}

dependencies {
    ktlint(libs.compose.rules)
}
