package org.jlleitschuh.gradle.ktlint

import org.gradle.api.file.FileTree
import org.gradle.api.tasks.TaskCollection
import org.gradle.api.tasks.TaskProvider
import org.gradle.api.tasks.util.PatternSet
import org.gradle.language.base.plugins.LifecycleBasePlugin
import org.jlleitschuh.gradle.ktlint.tasks.BaseKtLintCheckTask
import org.jlleitschuh.gradle.ktlint.tasks.GenerateBaselineTask
import org.jlleitschuh.gradle.ktlint.tasks.GenerateReportsTask
import org.jlleitschuh.gradle.ktlint.tasks.KtLintCheckTask
import org.jlleitschuh.gradle.ktlint.tasks.KtLintFormatTask
import org.jlleitschuh.gradle.ktlint.tasks.LoadReportersTask

internal fun KtlintPlugin.PluginHolder.addGenerateReportsTaskToProjectMetaCheckTask(
    generatesReportsTask: TaskProvider<GenerateReportsTask>
) {
    metaKtlintCheckTask.configure { it.dependsOn(generatesReportsTask) }
}

internal fun KtlintPlugin.PluginHolder.addGenerateReportsTaskToProjectMetaFormatTask(
    generateReportsTask: TaskProvider<GenerateReportsTask>
) {
    metaKtlintFormatTask.configure { it.dependsOn(generateReportsTask) }
}

/** A source set's check and format tasks with their report tasks, wired as upstream. */
internal fun KtlintPlugin.PluginHolder.addSourceSetTasks(sourceSetName: String, sourceDirectories: Iterable<*>) {
    val checkTask = createCheckTask(this, sourceSetName, sourceDirectories)
    val generateReportsCheckTask =
        createGenerateReportsTask(this, checkTask, GenerateReportsTask.LintType.CHECK, sourceSetName)
    addGenerateReportsTaskToProjectMetaCheckTask(generateReportsCheckTask)
    setCheckTaskDependsOnGenerateReportsTask(generateReportsCheckTask)

    val formatTask = createFormatTask(this, sourceSetName, sourceDirectories)
    val generateReportsFormatTask =
        createGenerateReportsTask(this, formatTask, GenerateReportsTask.LintType.FORMAT, sourceSetName)
    addGenerateReportsTaskToProjectMetaFormatTask(generateReportsFormatTask)
}

internal fun createFormatTask(
    pluginHolder: KtlintPlugin.PluginHolder,
    sourceSetName: String,
    kotlinSourceDirectories: Iterable<*>,
): TaskProvider<KtLintFormatTask> =
    pluginHolder.target.tasks.register(
        KtLintFormatTask.buildTaskNameForSourceSet(sourceSetName),
        KtLintFormatTask::class.java,
        PatternSet(),
    ).also { provider ->
        provider.configure {
            it.mustRunAfter(it.project.tasks.named(KtLintFormatTask.KOTLIN_SCRIPT_TASK_NAME))
            it.description = KtLintFormatTask.buildDescription(".kt")
            it.configureBaseCheckTask(pluginHolder)
            it.setSource(kotlinSourceDirectories)
        }
    }

internal fun createCheckTask(
    pluginHolder: KtlintPlugin.PluginHolder,
    sourceSetName: String,
    kotlinSourceDirectories: Iterable<*>,
): TaskProvider<KtLintCheckTask> =
    pluginHolder.target.tasks.register(
        KtLintCheckTask.buildTaskNameForSourceSet(sourceSetName),
        KtLintCheckTask::class.java,
        PatternSet(),
    ).also { provider ->
        provider.configure {
            it.description = KtLintCheckTask.buildDescription(".kt")
            it.configureBaseCheckTask(pluginHolder)
            it.setSource(kotlinSourceDirectories)
        }
    }

internal fun createKotlinScriptCheckTask(
    pluginHolder: KtlintPlugin.PluginHolder,
    projectScriptFiles: FileTree,
): TaskProvider<KtLintCheckTask> =
    pluginHolder.target.tasks.register(KtLintCheckTask.KOTLIN_SCRIPT_TASK_NAME, KtLintCheckTask::class.java, PatternSet())
        .also { provider ->
            provider.configure {
                it.description = KtLintCheckTask.buildDescription(".kts")
                it.configureBaseCheckTask(pluginHolder)
                it.setSource(projectScriptFiles)
            }
        }

internal fun createKotlinScriptFormatTask(
    pluginHolder: KtlintPlugin.PluginHolder,
    projectScriptFiles: FileTree,
): TaskProvider<KtLintFormatTask> =
    pluginHolder.target.tasks.register(KtLintFormatTask.KOTLIN_SCRIPT_TASK_NAME, KtLintFormatTask::class.java, PatternSet())
        .also { provider ->
            provider.configure {
                it.description = KtLintFormatTask.buildDescription(".kts")
                it.configureBaseCheckTask(pluginHolder)
                it.setSource(projectScriptFiles)
            }
        }

internal fun KtlintPlugin.PluginHolder.setCheckTaskDependsOnGenerateReportsTask(
    generateReportsTask: TaskProvider<GenerateReportsTask>
) {
    target.plugins.withType(LifecycleBasePlugin::class.java) {
        target.tasks.named(LifecycleBasePlugin.CHECK_TASK_NAME) { it.dependsOn(generateReportsTask) }
    }
}

internal fun createLoadReportersTask(pluginHolder: KtlintPlugin.PluginHolder): TaskProvider<LoadReportersTask> =
    pluginHolder.target.tasks.register(LoadReportersTask.TASK_NAME, LoadReportersTask::class.java) {
        it.description = LoadReportersTask.DESCRIPTION
        it.ktLintClasspath.setFrom(pluginHolder.ktlintConfiguration)
        it.reportersClasspath.setFrom(pluginHolder.ktlintReporterConfiguration)
        it.debug.set(pluginHolder.extension.debug)
        it.ktLintVersion.set(pluginHolder.extension.version)
        it.enabledReporters.set(pluginHolder.extension.reporterExtension.reporters)
        it.customReporters.set(pluginHolder.extension.reporterExtension.customReporters)
    }

private fun BaseKtLintCheckTask.configureBaseCheckTask(pluginHolder: KtlintPlugin.PluginHolder) {
    val extension = pluginHolder.extension
    ktLintClasspath.setFrom(pluginHolder.ktlintConfiguration)
    ktLintVersion.set(extension.version)
    additionalEditorconfig.set(extension.additionalEditorconfig)
    debug.set(extension.debug)
    ruleSetsClasspath.setFrom(pluginHolder.ruleSetJars)
    android.set(extension.android)
    loadedReporters.set(pluginHolder.loadReportersTask.flatMap { it.loadedReporters })
    enableExperimentalRules.set(extension.enableExperimentalRules)
    relative.set(extension.relative)
    coloredOutput.set(extension.coloredOutput)
    outputColorName.set(extension.outputColorName)
    baseline.set(extension.baseline.map { it.takeIf { file -> file.asFile.exists() } })
    ktrsVersion.set(pluginHolder.ktrsVersion)
    ktrsExecutable.set(pluginHolder.ktrsExecutable)
}

internal fun <T : BaseKtLintCheckTask> createGenerateReportsTask(
    pluginHolder: KtlintPlugin.PluginHolder,
    lintTask: TaskProvider<T>,
    lintType: GenerateReportsTask.LintType,
    sourceSetName: String,
): TaskProvider<GenerateReportsTask> =
    registerGenerateReportsTask(pluginHolder, lintTask, GenerateReportsTask.generateNameForSourceSets(sourceSetName, lintType))
        .also { provider ->
            provider.configure { it.mustRunAfter(it.project.tasks.named(KtLintFormatTask.KOTLIN_SCRIPT_TASK_NAME)) }
        }

internal fun <T : BaseKtLintCheckTask> createKotlinScriptGenerateReportsTask(
    pluginHolder: KtlintPlugin.PluginHolder,
    lintTask: TaskProvider<T>,
    lintType: GenerateReportsTask.LintType,
): TaskProvider<GenerateReportsTask> =
    registerGenerateReportsTask(pluginHolder, lintTask, GenerateReportsTask.generateNameForKotlinScripts(lintType))

private fun <T : BaseKtLintCheckTask> registerGenerateReportsTask(
    pluginHolder: KtlintPlugin.PluginHolder,
    lintTask: TaskProvider<T>,
    name: String,
): TaskProvider<GenerateReportsTask> =
    pluginHolder.target.tasks.register(name, GenerateReportsTask::class.java) {
        it.description = GenerateReportsTask.DESCRIPTION
        it.dependsOn(lintTask)
        it.reportsName.set(name)
        it.discoveredErrors.set(lintTask.flatMap { task -> task.discoveredErrors })
        it.lintReports.set(lintTask.flatMap { task -> task.reportsDirectory })
        it.loadedReporters.set(pluginHolder.loadReportersTask.flatMap { task -> task.loadedReporters })
        it.outputToConsole.set(pluginHolder.extension.outputToConsole)
        it.ignoreFailures.set(pluginHolder.extension.ignoreFailures)
        it.verbose.set(pluginHolder.extension.verbose)
        it.enabledReporters.set(pluginHolder.extension.reporterExtension.reporters)
    }

internal fun createGenerateBaselineTask(
    pluginHolder: KtlintPlugin.PluginHolder,
    lintTasks: TaskCollection<out BaseKtLintCheckTask>,
): TaskProvider<GenerateBaselineTask> =
    pluginHolder.target.tasks.register(GenerateBaselineTask.NAME, GenerateBaselineTask::class.java) {
        it.description = GenerateBaselineTask.DESCRIPTION
        it.group = HELP_GROUP
        it.dependsOn(lintTasks)
        it.ktLintClasspath.setFrom(pluginHolder.ktlintConfiguration)
        it.baselineReporterClasspath.setFrom(pluginHolder.ktlintBaselineReporterConfiguration)
        it.ruleSetsClasspath.setFrom(pluginHolder.ruleSetJars)
        it.sources.from({ lintTasks.map { task -> task.source } })
        it.ktLintVersion.set(pluginHolder.extension.version)
        it.additionalEditorconfig.set(pluginHolder.extension.additionalEditorconfig)
        it.ktrsVersion.set(pluginHolder.ktrsVersion)
        it.ktrsExecutable.set(pluginHolder.ktrsExecutable)
        it.debug.set(pluginHolder.extension.debug)
        it.baselineFile.set(pluginHolder.extension.baseline)
    }
