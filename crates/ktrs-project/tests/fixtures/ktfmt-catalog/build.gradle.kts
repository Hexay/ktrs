plugins {
    alias(libs.plugins.kotlin.jvm)
    alias(libs.plugins.ktfmt)
}

val width = 120

ktfmt {
    kotlinLangStyle()
    maxWidth.set(width)
    removeUnusedImports = false
    trailingCommaManagementStrategy.set(TrailingCommaManagementStrategy.NONE)
}
