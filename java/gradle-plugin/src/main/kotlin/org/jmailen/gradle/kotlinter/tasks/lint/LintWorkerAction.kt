package org.jmailen.gradle.kotlinter.tasks.lint

import io.github.hexay.ktrs.gradle.kotlinter.KotlinterCommand
import io.github.hexay.ktrs.gradle.kotlinter.WorkerEvent
import java.io.File
import org.gradle.api.logging.Logger
import org.gradle.api.logging.Logging
import org.gradle.workers.WorkAction
import org.jmailen.gradle.kotlinter.support.KotlinterError
import org.jmailen.gradle.kotlinter.support.LintFailure
import org.jmailen.gradle.kotlinter.support.ReporterType
import org.jmailen.gradle.kotlinter.support.resetEditorconfigCacheIfNeeded
import org.jmailen.gradle.kotlinter.tasks.LintTask
import org.jmailen.gradle.kotlinter.tasks.workerErrorMessage

public abstract class LintWorkerAction : WorkAction<LintWorkerParameters> {
    private val logger: Logger = Logging.getLogger(LintTask::class.java)

    // Upstream's `reporterFor`: an unknown name fails the action before any file is linted.
    private val reporters: Map<String, File> =
        parameters.reporters.get().onEach { (reporterName, _) -> ReporterType.valueOf(reporterName) }
    private val files: List<File> = parameters.files.toList()
    private val projectDirectory: File = parameters.projectDirectory.asFile.get()
    private val name: String = parameters.name.get()

    override fun execute() {
        resetEditorconfigCacheIfNeeded(changedEditorconfigFiles = parameters.changedEditorConfigFiles, logger = logger)
        var hasError = false

        var currentFile: File? = null
        try {
            files.filter { it.extension !in supportedExtensions }.forEach {
                logger.debug("$name ignoring non Kotlin file: ${it.toRelativeString(projectDirectory)}")
            }
            val command =
                KotlinterCommand(
                    parameters.ktlintVersion.get(),
                    parameters.ktlintClasspath,
                    parameters.temporaryDir.asFile.get(),
                    parameters.ktrsExecutable.orNull,
                    projectDirectory,
                    logger,
                )
            command.lint(files.filter { it.extension in supportedExtensions }, reporters).forEach { event ->
                if (event !is WorkerEvent.Error) return@forEach
                val error = event.error
                currentFile = File(error.file)
                if (event.isException) throw RuntimeException(error.message)
                hasError = true
                logger.error("${error.file}:${error.line}:${error.column}: Lint error > [${error.rule}] ${event.detail}")
            }
        } catch (t: Throwable) {
            throw KotlinterError(workerErrorMessage("lint", currentFile, t), t)
        }

        if (hasError) {
            throw LintFailure("kotlin source $name failed lint check")
        }
    }
}

private val supportedExtensions = setOf("kt", "kts")
