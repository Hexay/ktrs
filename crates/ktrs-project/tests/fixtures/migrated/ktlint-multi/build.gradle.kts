import org.jlleitschuh.gradle.ktlint.KtlintExtension

plugins {
    alias(libs.plugins.ktlint) apply false
}

subprojects {
    apply(plugin = "io.github.hexay.ktrs.ktlint")
    configure<KtlintExtension> {
        version.set("2.0.0-ALPHA-4")
        android.set(true)
        additionalEditorconfig.set(mapOf("max_line_length" to "120"))
    }
}
