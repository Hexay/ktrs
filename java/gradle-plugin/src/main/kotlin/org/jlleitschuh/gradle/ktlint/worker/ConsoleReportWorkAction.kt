package org.jlleitschuh.gradle.ktlint.worker

import io.github.hexay.ktrs.gradle.ktlint.RunErrors
import org.gradle.api.GradleException
import org.gradle.api.file.DirectoryProperty
import org.gradle.api.file.RegularFileProperty
import org.gradle.api.logging.Logging
import org.gradle.api.provider.ListProperty
import org.gradle.api.provider.Property
import org.gradle.workers.WorkAction
import org.gradle.workers.WorkParameters

/**
 * Upstream's console report: the errors not corrected, then the failure. A work action like upstream's, so the
 * failure reads "A failure occurred while executing …ConsoleReportWorkAction" as there.
 */
internal abstract class ConsoleReportWorkAction : WorkAction<ConsoleReportWorkAction.ConsoleReportParameters> {

    private val logger = Logging.getLogger("ktlint-console-report-worker")

    override fun execute() {
        // Distinct: a run handed to the ktlint jar reports an unfixable error once per format pass.
        val lintErrors = RunErrors.read(parameters.discoveredErrors.asFile.get()).filter { !it.corrected }.distinct()
        if (parameters.outputToConsole.get()) {
            val base = parameters.rootDirectory.asFile.get()
            val verbose = parameters.verbose.get()
            lintErrors.forEach {
                val verboseSuffix = if (verbose) " (${it.rule})" else ""
                logger.warn("${it.absolutePath(base)}:${it.line}:${it.column} ${it.consoleDetail}$verboseSuffix")
            }
        }

        if (!parameters.ignoreFailures.get() && lintErrors.isNotEmpty()) {
            val reportsPaths = parameters.generatedReportsPaths.get().joinToString(separator = "\n") { "- $it" }
            throw GradleException("KtLint found code style violations. Please see the following reports:\n$reportsPaths")
        }
    }

    internal interface ConsoleReportParameters : WorkParameters {
        val discoveredErrors: RegularFileProperty
        val outputToConsole: Property<Boolean>
        val ignoreFailures: Property<Boolean>
        val verbose: Property<Boolean>
        val generatedReportsPaths: ListProperty<String>
        val rootDirectory: DirectoryProperty
    }
}
