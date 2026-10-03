package io.github.hexay.ktrs.gradle.ktlint

import java.io.File
import org.gradle.api.GradleException
import org.gradle.api.logging.Logger
import org.jlleitschuh.gradle.ktlint.tasks.BaseKtLintCheckTask

/**
 * The `ktrs ktlint` command lines of the plugin's tasks. They run in the project directory, which
 * is what baseline paths and `relative` report paths are relative to.
 */
internal class KtlintCommand(
    private val version: String,
    private val ruleSetClasspath: Iterable<File>,
    private val additionalEditorconfig: Map<String, String>,
    private val ktrs: KtrsKtlint,
    private val tempDir: File,
) {

    constructor(
        task: BaseKtLintCheckTask
    ) : this(
        task.ktLintVersion.get(),
        task.ruleSetsClasspath,
        task.additionalEditorconfig.get(),
        KtrsKtlint(task.ktrsExecutable.orNull, task.projectDirectory, task.temporaryDir, task.logger, task.debug.get()),
        task.temporaryDir,
    )

    /**
     * Lints [files] (or formats them in place), with the task's reporters writing into [reportsDir]
     * and the `json` reporter into [consoleReport]. Fails like upstream when a file can't be parsed.
     */
    fun lint(task: BaseKtLintCheckTask, files: List<File>, format: Boolean, reportsDir: File, consoleReport: File) {
        val options = buildList {
            addAll(commonOptions())
            if (format) add("--format")
            if (task.relative.get()) add("--relative")
            if (task.coloredOutput.get()) add("--color")
            task.outputColorName.get().takeIf { it.isNotBlank() }?.let { add("--color-name=$it") }
            task.baseline.orNull?.asFile?.let { add("--baseline=${it.absolutePath}") }
        }
        val reporters =
            LoadedReporter.read(task.loadedReporters.get().asFile).map {
                it.cliOption(File(reportsDir, "report.${it.fileExtension}"))
            }
        ktrs.run(options + "--reporter=json,output=${consoleReport.absolutePath}" + reporters, files)
        if (!consoleReport.isFile) throw GradleException("ktrs ktlint wrote no report to $consoleReport")
        JsonReport.read(consoleReport).firstOrNull { it.rule.isEmpty() }?.let {
            throw GradleException("KtLint failed to parse file: ${it.absolutePath(task.projectDirectory)}\n${it.message}")
        }
    }

    /** Writes the baseline of [files] (every error, none filtered by an existing baseline) to [baselineFile]. */
    fun generateBaseline(files: List<File>, baselineFile: File) {
        ktrs.run(commonOptions() + "--reporter=baseline,output=${baselineFile.absolutePath}", files)
    }

    private fun commonOptions(): List<String> = buildList {
        add(KtlintVersions.cliOption(version))
        JarServices.ruleSetJars(ruleSetClasspath, version, tempDir).forEach { add("--ruleset=${it.absolutePath}") }
        if (additionalEditorconfig.isNotEmpty()) {
            val editorconfig = File(tempDir, "additional.editorconfig")
            editorconfig.parentFile.mkdirs()
            editorconfig.writeText(
                additionalEditorconfig.entries.joinToString("", prefix = "[*.{kt,kts}]\n") { "${it.key} = ${it.value}\n" }
            )
            add("--editorconfig=${editorconfig.absolutePath}")
        }
    }
}
