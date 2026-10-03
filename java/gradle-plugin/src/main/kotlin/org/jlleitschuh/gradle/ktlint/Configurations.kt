package org.jlleitschuh.gradle.ktlint

import org.gradle.api.Project
import org.gradle.api.artifacts.Configuration
import org.gradle.api.attributes.Bundling

internal const val KTLINT_CONFIGURATION_NAME = "ktlint"
internal const val KTLINT_CONFIGURATION_DESCRIPTION = "Main ktlint-gradle configuration"
internal const val KTLINT_RULESET_CONFIGURATION_NAME = "ktlintRuleset"
internal const val KTLINT_RULESET_CONFIGURATION_DESCRIPTION = "All ktlint rulesets dependencies"
internal const val KTLINT_REPORTER_CONFIGURATION_NAME = "ktlintReporter"
internal const val KTLINT_REPORTER_CONFIGURATION_DESCRIPTION = "All ktlint custom reporters dependencies"
internal const val KTLINT_BASELINE_REPORTER_CONFIGURATION_NAME = "ktlintBaselineReporter"
internal const val KTLINT_BASELINE_REPORTER_CONFIGURATION_DESCRIPTION =
    "Provides KtLint baseline reporter required to generate baseline file"

// Upstream's configurations, without the ktlint artifacts upstream adds to them: ktrs runs its own
// ktlint, so only user-added rule sets and reporters are resolved.

internal fun createKtlintConfiguration(target: Project): Configuration =
    target.configurations.maybeCreate(KTLINT_CONFIGURATION_NAME).apply {
        if (state != Configuration.State.UNRESOLVED) return@apply
        description = KTLINT_CONFIGURATION_DESCRIPTION
        isCanBeResolved = true
        isCanBeConsumed = false
        attributes {
            it.attribute(
                Bundling.BUNDLING_ATTRIBUTE,
                target.objects.named(Bundling::class.java, Bundling.EXTERNAL),
            )
        }
    }

internal fun createKtlintRulesetConfiguration(
    target: Project,
    ktLintConfiguration: Configuration,
): Configuration =
    target.configurations.maybeCreate(KTLINT_RULESET_CONFIGURATION_NAME).apply {
        description = KTLINT_RULESET_CONFIGURATION_DESCRIPTION
        isCanBeResolved = true
        isCanBeConsumed = false
        shouldResolveConsistentlyWith(ktLintConfiguration)
    }

internal fun createKtLintReporterConfiguration(
    target: Project,
    extension: KtlintExtension,
    ktLintConfiguration: Configuration,
): Configuration =
    target.configurations.maybeCreate(KTLINT_REPORTER_CONFIGURATION_NAME).apply {
        description = KTLINT_REPORTER_CONFIGURATION_DESCRIPTION
        isCanBeResolved = true
        isCanBeConsumed = false
        shouldResolveConsistentlyWith(ktLintConfiguration)
        withDependencies {
            extension.reporterExtension.customReporters.forEach { reporter ->
                dependencies.addLater(
                    target.provider {
                        val notation =
                            requireNotNull(reporter.dependency) {
                                "Reporter ${reporter.reporterId} dependency is not set!"
                            }
                        target.dependencies.create(notation)
                    }
                )
            }
        }
    }

internal fun createKtLintBaselineReporterConfiguration(
    target: Project,
    ktLintConfiguration: Configuration,
): Configuration =
    target.configurations.maybeCreate(KTLINT_BASELINE_REPORTER_CONFIGURATION_NAME).apply {
        description = KTLINT_BASELINE_REPORTER_CONFIGURATION_DESCRIPTION
        isCanBeResolved = true
        isCanBeConsumed = false
        shouldResolveConsistentlyWith(ktLintConfiguration)
    }
