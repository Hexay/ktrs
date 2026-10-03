package io.github.hexay.ktrs.gradle.ktlint

import org.gradle.api.GradleException

/** The ktlint versions ktrs implements, and what each one means on the `ktrs ktlint` command line. */
internal object KtlintVersions {
    const val V1_8 = "1.8.0"
    const val V2_0 = "2.0.0-ALPHA-4"

    /** The drop-in's `--ktlint-version` option for [version]; fails the build for any other version. */
    fun cliOption(version: String): String =
        when (version) {
            V1_8 -> "--ktlint-version=1.8"
            V2_0 -> "--ktlint-version=2.0"
            else ->
                throw GradleException(
                    "ktrs runs ktlint $V1_8 or $V2_0, not ktlint $version: set `ktlint { version }` to one " +
                        "of them, or use the org.jlleitschuh.gradle.ktlint plugin for other versions."
                )
        }

    /** The service interfaces a rule set JAR implements for [version] (`-R` loads these). */
    fun ruleSetInterfaces(version: String): List<String> =
        if (version == V1_8) listOf(RULE_SET_PROVIDER_V3) else listOf(RULE_SET_V2_PROVIDER, RULE_SET_PROVIDER_V3)

    /** The service interface a reporter JAR implements for [version]. */
    fun reporterInterface(version: String): String =
        if (version == V1_8) "com.pinterest.ktlint.cli.reporter.core.api.ReporterProviderV2"
        else "io.github.ktlint.core.cli.reporter.core.api.ReporterProviderV2"

    private const val RULE_SET_PROVIDER_V3 = "com.pinterest.ktlint.cli.ruleset.core.api.RuleSetProviderV3"
    private const val RULE_SET_V2_PROVIDER = "io.github.ktlint.core.cli.ruleset.core.api.RuleSetV2Provider"
}
