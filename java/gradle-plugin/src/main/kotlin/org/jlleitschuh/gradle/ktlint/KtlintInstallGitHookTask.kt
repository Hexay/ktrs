package org.jlleitschuh.gradle.ktlint

import java.io.File
import javax.inject.Inject
import org.gradle.api.DefaultTask
import org.gradle.api.file.ProjectLayout
import org.gradle.api.model.ObjectFactory
import org.gradle.api.provider.Property
import org.gradle.api.tasks.Input
import org.gradle.api.tasks.TaskAction
import org.gradle.api.tasks.UntrackedTask

internal fun KtlintPlugin.PluginHolder.addGitHookTasks() {
    if (target.rootProject == target) {
        target.tasks.register(INSTALL_GIT_HOOK_FORMAT_TASK, KtlintInstallGitHookTask::class.java) {
            it.description = "Adds git hook to run ktlintFormat on changed files"
            it.group = HELP_GROUP
            it.taskName.set(FORMAT_PARENT_TASK_NAME)
            // Format git hook will automatically add back updated files to git commit
            it.shouldUpdateCommit.set(true)
            it.hookName.set("pre-commit")
        }
        target.tasks.register(INSTALL_GIT_HOOK_CHECK_TASK, KtlintInstallGitHookTask::class.java) {
            it.description = "Adds git hook to run ktlintCheck on changed files"
            it.group = HELP_GROUP
            it.taskName.set(CHECK_PARENT_TASK_NAME)
            it.shouldUpdateCommit.set(false)
            it.hookName.set("pre-commit")
        }
    }
}

@UntrackedTask(because = "Utility task used to install git hooks. Often only run once.")
public open class KtlintInstallGitHookTask
@Inject
constructor(objectFactory: ObjectFactory, projectLayout: ProjectLayout) : DefaultTask() {
    @get:Input internal val taskName: Property<String> = objectFactory.property(String::class.java)

    @get:Input
    internal val shouldUpdateCommit: Property<Boolean> =
        objectFactory.property(Boolean::class.java).convention(false)

    @get:Input internal val hookName: Property<String> = objectFactory.property(String::class.java)

    // Paths as strings: the hook depends on where the directories are, not on their contents.
    @get:Input
    internal val projectDir: Property<String> =
        objectFactory.property(String::class.java).value(projectLayout.projectDirectory.asFile.absolutePath)

    @get:Input
    internal val rootDirectory: Property<String> =
        objectFactory.property(String::class.java).value(project.rootDir.absolutePath)

    @TaskAction
    public fun installHook() {
        val (gitDir, workTree) =
            findGitDir(File(projectDir.get()))?.takeIf { File(it.first, "objects").exists() }
                ?: run {
                    logger.warn("No git folder was found!")
                    return
                }

        logger.info(".git directory path: $gitDir")
        val gitHookDirectory = gitDir.resolve("hooks")
        if (!gitHookDirectory.exists()) {
            logger.info("git hooks directory doesn't exist, creating one")
            gitHookDirectory.mkdir()
        }

        val gitHookFile = gitDir.resolve("hooks/${hookName.get()}")
        logger.info("Hook file: $gitHookFile")
        if (!gitHookFile.exists()) {
            gitHookFile.createNewFile()
            gitHookFile.setExecutable(true)
        }
        val gradleRootDirPrefix = File(rootDirectory.get()).relativeTo(workTree).path
        val hook = "$startHookSection${generateGitHook(taskName.get(), shouldUpdateCommit.get(), gradleRootDirPrefix)}"

        if (gitHookFile.length() == 0L) {
            gitHookFile.writeText("$shShebang$hook$endHookSection")
            return
        }

        val hookContent = gitHookFile.readText()
        if (hookContent.contains(startHookSection)) {
            val startTagIndex = hookContent.indexOf(startHookSection)
            val endTagIndex = hookContent.indexOf(endHookSection)
            gitHookFile.writeText(hookContent.replaceRange(startTagIndex, endTagIndex, hook))
        } else {
            gitHookFile.appendText("$hook$endHookSection")
        }
    }
}
