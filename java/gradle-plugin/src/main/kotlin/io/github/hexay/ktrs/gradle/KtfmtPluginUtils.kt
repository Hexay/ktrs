package io.github.hexay.ktrs.gradle

import com.ncorti.ktfmt.gradle.KtfmtExtension
import com.ncorti.ktfmt.gradle.tasks.KtfmtCheckTask
import com.ncorti.ktfmt.gradle.tasks.KtfmtFormatTask
import java.io.File
import org.gradle.api.Project
import org.gradle.api.Task
import org.gradle.api.file.FileCollection
import org.gradle.api.provider.Provider
import org.gradle.api.tasks.TaskProvider
import org.gradle.language.base.plugins.LifecycleBasePlugin

/** Port of ktfmt-gradle's `KtfmtPluginUtils`: the per-source-set and scripts tasks. */
internal object KtfmtPluginUtils {

    internal const val EXTENSION_NAME = "ktfmt"

    internal const val TASK_NAME_FORMAT = "ktfmtFormat"

    internal const val TASK_NAME_CHECK = "ktfmtCheck"

    internal val defaultIncludes = listOf("**/*.kt", "**/*.kts")

    private const val KOTLIN_COMPILE = "org.jetbrains.kotlin.gradle.tasks.KotlinCompile"

    internal fun shouldCreateTasks(srcSetName: String): Boolean =
        when {
            // KSP's generated-code source sets (`generatedByKspKotlin`, `generatedByKspTestKotlin`).
            srcSetName.startsWith("generatedByKsp") -> false
            // Spring AOT's.
            srcSetName == "aot" || srcSetName == "aotTest" -> false
            else -> true
        }

    @Suppress("LongParameterList")
    internal fun createTasksForSourceSet(
        project: Project,
        srcSetName: String,
        srcSetDir: FileCollection,
        ktfmtExtension: KtfmtExtension,
        topLevelFormat: TaskProvider<Task>,
        topLevelCheck: TaskProvider<Task>,
    ) {
        if (shouldCreateTasks(srcSetName).not()) {
            return
        }

        val srcCheckTask = createCheckTask(project, srcSetName, srcSetDir, ktfmtExtension)
        val srcFormatTask = createFormatTask(project, srcSetName, srcSetDir, ktfmtExtension)

        wireTasks(project, srcCheckTask, srcFormatTask, topLevelFormat, topLevelCheck)
    }

    internal fun createScriptsTasks(
        project: Project,
        projectDir: File,
        topLevelFormat: TaskProvider<Task>,
        topLevelCheck: TaskProvider<Task>,
    ) {
        val scriptFiles =
            project
                .fileTree(projectDir)
                .filter { it.extension == "kts" }
                .filter { it.parentFile == projectDir }

        val scriptCheckTask =
            project.tasks.register("${TASK_NAME_CHECK}Scripts", KtfmtCheckTask::class.java) {
                it.description =
                    "Run Ktfmt formatter validation for script files on project '${project.name}'"
                it.setSource(scriptFiles)
                it.setIncludes(defaultIncludes)
            }
        val scriptFormatTask =
            project.tasks.register("${TASK_NAME_FORMAT}Scripts", KtfmtFormatTask::class.java) {
                it.description = "Run Ktfmt formatter for script files on project '${project.name}'"
                it.setSource(scriptFiles)
                it.setIncludes(defaultIncludes)
            }

        wireTasks(project, scriptCheckTask, scriptFormatTask, topLevelFormat, topLevelCheck)
    }

    private fun wireTasks(
        project: Project,
        checkTask: TaskProvider<KtfmtCheckTask>,
        formatTask: TaskProvider<KtfmtFormatTask>,
        topLevelFormat: TaskProvider<Task>,
        topLevelCheck: TaskProvider<Task>,
    ) {
        // ktfmt tasks edit the sources, so they go before compileKotlin. Matched by name: the
        // Kotlin plugin may not be on this plugin's classpath (the scripts tasks exist without it).
        project.tasks.configureEach { task ->
            if (task.isKotlinCompile()) task.mustRunAfter(checkTask, formatTask)
        }

        topLevelFormat.configure { task -> task.dependsOn(formatTask) }
        topLevelCheck.configure { task -> task.dependsOn(checkTask) }

        project.plugins.withType(LifecycleBasePlugin::class.java).configureEach {
            project.tasks.named(LifecycleBasePlugin.CHECK_TASK_NAME).configure { task ->
                task.dependsOn(checkTask)
            }
        }
    }

    private fun Task.isKotlinCompile(): Boolean =
        generateSequence<Class<*>>(javaClass) { it.superclass }.any { it.name == KOTLIN_COMPILE }

    private fun createCheckTask(
        project: Project,
        name: String,
        srcDir: FileCollection,
        ktfmtExtension: KtfmtExtension,
    ): TaskProvider<KtfmtCheckTask> {
        val inputDirs = project.getSelectedSrcSets(srcDir, ktfmtExtension)
        return project.tasks.register(
            "$TASK_NAME_CHECK${capitalize(name)}",
            KtfmtCheckTask::class.java,
        ) {
            it.description =
                "Run Ktfmt formatter validation for sourceSet '$name' on project '${project.name}'"
            it.setSource(inputDirs)
            it.setIncludes(defaultIncludes)
        }
    }

    private fun createFormatTask(
        project: Project,
        name: String,
        srcDir: FileCollection,
        ktfmtExtension: KtfmtExtension,
    ): TaskProvider<KtfmtFormatTask> {
        val inputDirs = project.getSelectedSrcSets(srcDir, ktfmtExtension)
        return project.tasks.register(
            "$TASK_NAME_FORMAT${capitalize(name)}",
            KtfmtFormatTask::class.java,
        ) {
            it.description =
                "Run Ktfmt formatter for sourceSet '$name' on project '${project.name}'"
            it.setSource(inputDirs)
            it.setIncludes(defaultIncludes)
        }
    }

    /** `"kmp commonMain"` -> `"KmpCommonMain"`. */
    private fun capitalize(name: String): String =
        name.split(" ").joinToString("") {
            val charArray = it.toCharArray()
            if (charArray[0].isLowerCase()) {
                charArray[0] = charArray[0].uppercaseChar()
            }
            charArray.concatToString()
        }

    private fun Project.getSelectedSrcSets(
        srcDir: FileCollection,
        ktfmtExtension: KtfmtExtension,
    ): Provider<List<File>> {
        val excludedSourceSets = ktfmtExtension.srcSetPathExclusionPattern

        return provider {
            srcDir.toList().filterNot { it.absolutePath.matches(excludedSourceSets.get()) }
        }
    }
}
