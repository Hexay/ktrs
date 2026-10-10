package org.jmailen.gradle.kotlinter

import org.jmailen.gradle.kotlinter.support.ReporterType
import org.jmailen.gradle.kotlinter.support.versionProperties

/** kotlinter's `kotlinter { }` block. [ktlintVersion]: "1.8.0" (default) or "2.0.0-ALPHA-4", the two ktrs runs. */
public open class KotlinterExtension {
    public companion object {
        public const val DEFAULT_IGNORE_FORMAT_FAILURES: Boolean = true
        public const val DEFAULT_IGNORE_LINT_FAILURES: Boolean = false
        public val DEFAULT_REPORTER: String = ReporterType.checkstyle.name
    }

    public var ktlintVersion: String = versionProperties.ktlintVersion()
    public var ignoreFormatFailures: Boolean = DEFAULT_IGNORE_FORMAT_FAILURES
    public var ignoreLintFailures: Boolean = DEFAULT_IGNORE_LINT_FAILURES
    public var reporters: Array<String> = arrayOf(DEFAULT_REPORTER)
}
