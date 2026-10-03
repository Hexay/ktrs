package org.jlleitschuh.gradle.ktlint

import java.io.File
import java.nio.file.Files
import java.nio.file.Path
import org.gradle.api.file.ProjectLayout
import org.gradle.api.file.RegularFile
import org.gradle.api.plugins.HelpTasksPlugin
import org.gradle.api.provider.Provider
import org.gradle.language.base.plugins.LifecycleBasePlugin

internal const val EDITOR_CONFIG_FILE_NAME = ".editorconfig"

internal fun getEditorConfigFiles(currentProjectDir: Path): Set<Path> {
    val result = mutableSetOf<Path>()
    searchEditorConfigFiles(currentProjectDir, result)
    return result
}

private tailrec fun searchEditorConfigFiles(projectPath: Path, result: MutableSet<Path>) {
    val editorConfigFC = projectPath.resolve(EDITOR_CONFIG_FILE_NAME)
    if (Files.exists(editorConfigFC)) {
        result.add(editorConfigFC.toAbsolutePath())
    }

    val parentDir = projectPath.parent
    if (parentDir != null && !editorConfigFC.isRootEditorConfig()) {
        searchEditorConfigFiles(parentDir, result)
    }
}

private val editorConfigRootRegex = "^root\\s?=\\s?true".toRegex()

internal fun Path.isRootEditorConfig(): Boolean {
    if (!Files.exists(this) || !Files.isReadable(this)) return false

    toFile().useLines { lines ->
        return lines.firstOrNull { it.contains(editorConfigRootRegex) } != null
    }
}

internal const val VERIFICATION_GROUP = LifecycleBasePlugin.VERIFICATION_GROUP
internal const val FORMATTING_GROUP = "Formatting"
internal const val HELP_GROUP = HelpTasksPlugin.HELP_GROUP
internal const val CHECK_PARENT_TASK_NAME = "ktlintCheck"
internal const val FORMAT_PARENT_TASK_NAME = "ktlintFormat"
internal const val INSTALL_GIT_HOOK_CHECK_TASK = "addKtlintCheckGitPreCommitHook"
internal const val INSTALL_GIT_HOOK_FORMAT_TASK = "addKtlintFormatGitPreCommitHook"
internal val KOTLIN_EXTENSIONS = listOf("kt", "kts")
internal val INTERMEDIATE_RESULTS_PATH = "intermediates${File.separator}ktLint${File.separator}"

/** Where tasks put intermediate results that other plugin tasks consume. */
internal fun ProjectLayout.intermediateResultsBuildDir(resultsFile: String): Provider<RegularFile> =
    buildDirectory.file("$INTERMEDIATE_RESULTS_PATH$resultsFile")

/** Kotlin's deprecated `String.capitalize()`, which upstream's task names use. */
internal fun String.capitalizeName(): String =
    replaceFirstChar { if (it.isLowerCase()) it.titlecase() else it.toString() }
