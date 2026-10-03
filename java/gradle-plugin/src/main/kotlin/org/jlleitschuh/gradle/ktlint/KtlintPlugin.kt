package org.jlleitschuh.gradle.ktlint

import io.github.hexay.ktrs.gradle.KtrsBuildService
import io.github.hexay.ktrs.gradle.ktlint.JarServices
import org.gradle.api.Plugin
import org.gradle.api.Project
import org.gradle.api.Task
import org.gradle.api.artifacts.Configuration
import org.gradle.api.file.FileCollection
import org.gradle.api.plugins.PluginContainer
import org.gradle.api.provider.Provider
import org.gradle.api.tasks.TaskProvider
import org.jlleitschuh.gradle.ktlint.android.applyKtLintToAndroid
import org.jlleitschuh.gradle.ktlint.tasks.GenerateReportsTask
import org.jlleitschuh.gradle.ktlint.tasks.KtLintCheckTask
import org.jlleitschuh.gradle.ktlint.tasks.LoadReportersTask

/**
 * `io.github.hexay.ktrs.ktlint`: ktlint-gradle 14.2.0's `KtlintPlugin` (same extension, tasks,
 * configurations and wiring), linting through the `ktrs ktlint` drop-in instead of ktlint in Gradle
 * workers. Deviations: research/29-ktlint-gradle-dropin.md.
 */
public open class KtlintPlugin : Plugin<Project> {

    override fun apply(target: Project) {
        val holder = PluginHolder(target)
        holder.addKotlinScriptTasks()
        holder.addKtLintTasksToKotlinPlugin()
        holder.addGenerateBaselineTask()
        holder.addGitHookTasks()
    }

    private fun PluginHolder.addKtLintTasksToKotlinPlugin() {
        val kotlin = { KotlinSourceSetsApplier(this) }
        target.plugins.withId("kotlin") { kotlin().applyKtLint() }
        target.plugins.withId("org.jetbrains.kotlin.js") { kotlin().applyKtLint() }
        target.plugins.withId("org.jetbrains.kotlin.multiplatform") { kotlin().applyKtlintMultiplatform() }
        addKtLintTasksToAndroidIfNecessary()
    }

    private fun PluginHolder.addKtLintTasksToAndroidIfNecessary() {
        // Classic API (with kotlin-android plugin)
        target.plugins.withId("org.jetbrains.kotlin.android") { applyKtLintToAndroid() }

        // New API from AGP 9+ (with built-in Kotlin plugin)
        target.pluginManager.withPlugin("com.android.base") {
            if (target.plugins.hasKotlinBaseApiPlugin) {
                target.plugins.withId(it.id) { applyKtLintToAndroid() }
            }
        }
    }

    @Suppress("UNCHECKED_CAST")
    private val kotlinBaseApiPluginClass by lazy {
        try {
            Class.forName("org.jetbrains.kotlin.gradle.plugin.KotlinBaseApiPlugin") as? Class<out Plugin<*>>
        } catch (_: ClassNotFoundException) {
            null
        }
    }

    private val PluginContainer.hasKotlinBaseApiPlugin: Boolean
        get() = kotlinBaseApiPluginClass?.let(this::hasPlugin) ?: false

    private fun PluginHolder.addKotlinScriptTasks() {
        val projectDirectoryScriptFiles = target.fileTree(target.projectDir)
        projectDirectoryScriptFiles.include("*.kts")

        val checkTask = createKotlinScriptCheckTask(this, projectDirectoryScriptFiles)
        val generateReportsCheckTask =
            createKotlinScriptGenerateReportsTask(this, checkTask, GenerateReportsTask.LintType.CHECK)
        addGenerateReportsTaskToProjectMetaCheckTask(generateReportsCheckTask)
        setCheckTaskDependsOnGenerateReportsTask(generateReportsCheckTask)

        val formatTask = createKotlinScriptFormatTask(this, projectDirectoryScriptFiles)
        val generateReportsFormatTask =
            createKotlinScriptGenerateReportsTask(this, formatTask, GenerateReportsTask.LintType.FORMAT)
        addGenerateReportsTaskToProjectMetaFormatTask(generateReportsFormatTask)
    }

    private fun PluginHolder.addGenerateBaselineTask() {
        createGenerateBaselineTask(this, target.tasks.withType(KtLintCheckTask::class.java))
    }

    internal class PluginHolder(val target: Project) {
        val extension: KtlintExtension = target.plugins.apply(KtlintBasePlugin::class.java).extension

        val metaKtlintCheckTask: TaskProvider<Task> by lazy {
            target.tasks.register(CHECK_PARENT_TASK_NAME) {
                it.group = VERIFICATION_GROUP
                it.description = "Runs ktlint on all kotlin sources in this project."
            }
        }

        val metaKtlintFormatTask: TaskProvider<Task> by lazy {
            target.tasks.register(FORMAT_PARENT_TASK_NAME) {
                it.group = FORMATTING_GROUP
                it.description = "Runs the ktlint formatter on all kotlin sources in this project."
            }
        }

        val ktlintConfiguration: Configuration = createKtlintConfiguration(target)
        val ktlintRulesetConfiguration: Configuration =
            createKtlintRulesetConfiguration(target, ktlintConfiguration)
        val ruleSetJars: FileCollection = JarServices.withoutKtlintItself(ktlintRulesetConfiguration)
        val ktlintReporterConfiguration: Configuration =
            createKtLintReporterConfiguration(target, extension, ktlintConfiguration)
        val ktlintBaselineReporterConfiguration: Configuration =
            createKtLintBaselineReporterConfiguration(target, ktlintConfiguration)

        val ktrsVersion: String =
            KtlintPlugin::class.java.getResource("/io/github/hexay/ktrs/gradle/ktrs-version.txt")?.readText()
                ?: error("Missing ktrs version")

        /** `-Pktrs.executable` / `-Dktrs.executable`, as for the ktfmt plugin ([KtrsBuildService]). */
        val ktrsExecutable: Provider<String> =
            target.providers
                .gradleProperty(KtrsBuildService.EXECUTABLE_PROPERTY)
                .orElse(target.providers.systemProperty(KtrsBuildService.EXECUTABLE_PROPERTY))

        val loadReportersTask: TaskProvider<LoadReportersTask> = createLoadReportersTask(this)
    }
}
