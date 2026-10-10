package io.github.hexay.ktrs.gradle.ktlint

import io.github.hexay.ktrs.KtlintJars
import org.gradle.api.GradleException

/** The ktlint versions ktrs implements, as `ktrs ktlint` options. */
internal object KtlintVersions {
    const val V1_8 = KtlintJars.V1_8
    const val V2_0 = KtlintJars.V2_0

    /**
     * The drop-in's `--ktlint-version` option for [version]; fails the build for any other version, naming the
     * build script [setting] that chose it and the [upstreamPlugin] that runs other versions.
     */
    fun cliOption(
        version: String,
        setting: String = "ktlint { version }",
        upstreamPlugin: String = "org.jlleitschuh.gradle.ktlint",
    ): String =
        when (version) {
            V1_8 -> "--ktlint-version=1.8"
            V2_0 -> "--ktlint-version=2.0"
            else ->
                throw GradleException(
                    "ktrs runs ktlint $V1_8 or $V2_0, not ktlint $version: set `$setting` to one " +
                        "of them, or use the $upstreamPlugin plugin for other versions."
                )
        }
}
