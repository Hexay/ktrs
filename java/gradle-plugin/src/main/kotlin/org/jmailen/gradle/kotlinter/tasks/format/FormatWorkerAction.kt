package org.jmailen.gradle.kotlinter.tasks.format

import io.github.hexay.ktrs.gradle.kotlinter.KotlinterCommand
import io.github.hexay.ktrs.gradle.kotlinter.WorkerEvent
import java.io.File
import org.gradle.api.logging.Logger
import org.gradle.api.logging.Logging
import org.gradle.workers.WorkAction
import org.jmailen.gradle.kotlinter.support.KotlinterError
import org.jmailen.gradle.kotlinter.support.LintFailure
import org.jmailen.gradle.kotlinter.support.resetEditorconfigCacheIfNeeded
import org.jmailen.gradle.kotlinter.tasks.FormatTask
import org.jmailen.gradle.kotlinter.tasks.workerErrorMessage

public abstract class FormatWorkerAction : WorkAction<FormatWorkerParameters> {
    private val logger: Logger = Logging.getLogger(FormatTask::class.java)
    private val files: List<File> = parameters.files.toList()
    private val projectDirectory: File = parameters.projectDirectory.asFile.get()
    private val name: String = parameters.name.get()
    private val output: File? = parameters.output.asFile.orNull

    override fun execute() {
        resetEditorconfigCacheIfNeeded(changedEditorconfigFiles = parameters.changedEditorConfigFiles, logger = logger)
        val fixes = mutableListOf<String>()

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
            command.format(files.filter { it.extension in supportedExtensions }).forEach { event ->
                when (event) {
                    is WorkerEvent.Formatted -> logger.warn("${event.file}: Format fixed")
                    is WorkerEvent.Error -> {
                        val error = event.error
                        currentFile = File(error.file)
                        if (event.isException) throw RuntimeException(error.message)
                        val position = "${error.file}:${error.line}:${error.column}"
                        val msg =
                            when (event.canBeAutoCorrected) {
                                true -> "$position: Format fixed > [${error.rule}] ${event.detail}"
                                false -> "$position: Format could not fix > [${error.rule}] ${event.detail}"
                            }
                        if (event.canBeAutoCorrected) {
                            logger.warn(msg)
                        } else {
                            hasError = true
                            logger.error(msg)
                        }
                        fixes.add(msg)
                    }
                }
            }
        } catch (t: Throwable) {
            throw KotlinterError(workerErrorMessage("format", currentFile, t), t)
        }

        if (hasError) {
            throw LintFailure("kotlin source $name failed lint check")
        }

        output?.writeText(
            when (fixes.isEmpty()) {
                true -> "ok"
                false -> fixes.joinToString("\n")
            }
        )
    }
}

private val supportedExtensions = setOf("kt", "kts")
