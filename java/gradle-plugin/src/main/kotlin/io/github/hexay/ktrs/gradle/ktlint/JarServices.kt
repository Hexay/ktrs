package io.github.hexay.ktrs.gradle.ktlint

import io.github.hexay.ktrs.KtlintJars
import org.gradle.api.artifacts.Configuration
import org.gradle.api.artifacts.component.ModuleComponentIdentifier
import org.gradle.api.file.FileCollection

/** Resolved `ktlintRuleset` / `ktlintReporter` files, turned into `ktrs ktlint` JARs by [KtlintJars]. */
internal object JarServices {

    /** [configuration]'s files without ktlint and its runtime (Kotlin, logging, ec4j). */
    fun withoutKtlintItself(configuration: Configuration): FileCollection =
        configuration.incoming
            .artifactView { view ->
                view.componentFilter { id -> !(id is ModuleComponentIdentifier && KtlintJars.isKtlintRuntime(id.group)) }
            }
            .files
}
