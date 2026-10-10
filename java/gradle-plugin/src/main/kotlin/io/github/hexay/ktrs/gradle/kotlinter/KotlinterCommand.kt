package io.github.hexay.ktrs.gradle.kotlinter

import io.github.hexay.ktrs.KtlintJars
import io.github.hexay.ktrs.gradle.ktlint.KtlintVersions
import io.github.hexay.ktrs.gradle.ktlint.KtrsKtlint
import io.github.hexay.ktrs.gradle.ktlint.ReportedError
import io.github.hexay.ktrs.gradle.ktlint.RunErrors
import java.io.File
import org.gradle.api.GradleException
import org.gradle.api.logging.Logger

/** What kotlinter's worker actions log for a file, in order. */
internal sealed interface WorkerEvent {
    /** An engine callback, or (see [isException]) what the engine threw. */
    data class Error(val error: ReportedError) : WorkerEvent {
        /** A run handed to the ktlint jar reports no statuses; a file it could not lint has no rule. */
        val isException: Boolean
            get() = error.status?.endsWith("_EXCEPTION") ?: error.rule.isEmpty()

        val canBeAutoCorrected: Boolean
            get() = error.status == "LINT_CAN_BE_AUTOCORRECTED"

        /** ktlint's `LintError.detail`: the jar's CLI adds a suffix to what `--format` could not fix. */
        val detail: String
            get() = if (error.status == null) error.message.removeSuffix(CANNOT_BE_AUTOCORRECTED) else error.message
    }

    /** The file's text changed. */
    data class Formatted(val file: String) : WorkerEvent
}

private const val CANNOT_BE_AUTOCORRECTED = " (cannot be auto-corrected)"

/**
 * The `ktrs ktlint` runs of kotlinter's worker actions, in kotlinter mode (ktrs's hidden option,
 * `crates/ktrs-cli/src/ktlint/kotlinter.rs`). They run in the project directory, which report paths are relative
 * to. A run that loads a rule set ktrs can't run natively is ktlint's CLI over its jar: its reports and events
 * follow the CLI (research/34, deviations).
 */
internal class KotlinterCommand(
    private val ktlintVersion: String,
    private val ruleSetClasspath: Iterable<File>,
    private val tempDir: File,
    executable: String?,
    private val projectDirectory: File,
    logger: Logger,
) {
    private val ktrs = KtrsKtlint(executable, projectDirectory, tempDir, logger, debug = false)

    /** Lints [files], each of [reporters] (kotlinter's name to its file) inside kotlinter's sorting wrapper. */
    fun lint(files: List<File>, reporters: Map<String, File>): List<WorkerEvent> =
        run(reporters.map { (name, output) -> "--reporter=$name,output=${output.absolutePath}" }, files)

    /** Formats [files] in place. */
    fun format(files: List<File>): List<WorkerEvent> = run(listOf("--format"), files)

    private fun run(options: List<String>, files: List<File>): List<WorkerEvent> {
        val events = File(tempDir, "events.txt")
        events.delete()
        val command = buildList {
            add(versionOption(ktlintVersion))
            // Without files the rule sets don't matter, and the ktlint jar would lint its default patterns.
            if (files.isNotEmpty()) {
                KtlintJars.ruleSetJars(ruleSetClasspath.toList(), ktlintVersion, tempDir).forEach {
                    add("--ruleset=${it.absolutePath}")
                }
            }
            addAll(options)
            // For a run handed to the ktlint jar: its reports get kotlinter's project-relative paths.
            add("--relative")
            add("--ktrs-kotlinter-events=${events.absolutePath}")
        }
        ktrs.run(command, files, allowNoFiles = true)
        if (!events.isFile) throw GradleException("ktrs ktlint wrote no events file $events")
        return read(events)
    }

    private fun read(events: File): List<WorkerEvent> {
        val lines = events.readLines()
        if (!RunErrors.isEvents(lines)) {
            return RunErrors.read(events).map { WorkerEvent.Error(it.copy(file = it.absolutePath(projectDirectory))) }
        }
        return lines.mapNotNull { line ->
            when {
                line.startsWith("error\t") -> WorkerEvent.Error(RunErrors.eventError(line))
                line.startsWith("formatted\t") -> WorkerEvent.Formatted(line.substringAfter('\t'))
                else -> null
            }
        }
    }

    companion object {
        /** `ktrs ktlint`'s option for `kotlinter { ktlintVersion }`; fails the build for a version ktrs does not run. */
        fun versionOption(ktlintVersion: String): String =
            KtlintVersions.cliOption(ktlintVersion, "kotlinter { ktlintVersion }", "org.jmailen.kotlinter")
    }
}
