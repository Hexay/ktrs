package io.github.hexay.ktrs.gradle.ktlint

import java.io.File
import org.gradle.api.GradleException
import org.jlleitschuh.gradle.ktlint.tasks.BaseKtLintCheckTask

/**
 * The `ktrs ktlint` command lines of the plugin's tasks, in ktlint-gradle mode (ktrs's hidden options,
 * `crates/ktrs-cli/src/ktlint/gradle.rs`). They run in the project directory, which baseline paths are
 * relative to; `relative` report paths are relative to the root project, as upstream's.
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
     * Lints [files] (or formats them in place), with the task's reporters writing into [reportsDir] and every
     * error into [errors]. Fails like upstream when a file can't be parsed.
     */
    fun lint(task: BaseKtLintCheckTask, files: List<File>, format: Boolean, reportsDir: File, errors: File) {
        val options = buildList {
            addAll(commonOptions())
            if (format) add("--format")
            if (task.relative.get()) {
                add("--relative")
                add("--ktrs-relative-to=${task.rootDirectory.absolutePath}")
            }
            if (task.coloredOutput.get()) add("--color")
            task.outputColorName.get().takeIf { it.isNotBlank() }?.let { add("--color-name=$it") }
            task.baseline.orNull?.asFile?.let { add("--baseline=${it.absolutePath}") }
            add("--ktrs-gradle-events=${errors.absolutePath}")
        }
        val reporters =
            LoadedReporter.read(task.loadedReporters.get().asFile).map {
                it.cliOption(File(reportsDir, "report.${it.fileExtension}"))
            }
        ktrs.run(options + reporters, files)
        if (!errors.isFile) throw GradleException("ktrs ktlint wrote no errors file $errors")
        RunErrors.read(errors).firstOrNull { it.rule.isEmpty() }?.let {
            throw GradleException("KtLint failed to parse file: ${it.absolutePath(task.reportPathBase)}\n${it.message}")
        }
    }

    /** Writes the baseline of [files] (every error, none filtered by an existing baseline) to [baselineFile]. */
    fun generateBaseline(files: List<File>, baselineFile: File) {
        ktrs.run(commonOptions() + "--reporter=baseline,output=${baselineFile.absolutePath}", files)
    }

    private fun commonOptions(): List<String> = buildList {
        add(KtlintVersions.cliOption(version))
        JarServices.ruleSetJars(ruleSetClasspath, version, tempDir).forEach { add("--ruleset=${it.absolutePath}") }
        additionalEditorconfig.forEach { (name, value) -> add("--ktrs-editorconfig-override=$name=$value") }
    }
}
