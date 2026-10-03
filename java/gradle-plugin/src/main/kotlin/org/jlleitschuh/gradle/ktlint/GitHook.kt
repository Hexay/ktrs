package org.jlleitschuh.gradle.ktlint

import java.io.File
import org.jlleitschuh.gradle.ktlint.tasks.BaseKtLintCheckTask

internal const val FILTER_INCLUDE_PROPERTY_NAME = "internalKtlintGitFilter"

internal val shShebang =
    """
    #!/bin/sh

    """
        .trimIndent()

internal const val startHookSection = "######## KTLINT-GRADLE HOOK START ########\n"
internal const val endHookSection = "######## KTLINT-GRADLE HOOK END ########\n"

private fun generateGradleCommand(taskName: String, gradleRootDirPrefix: String): String {
    val gradleCommand =
        if (gradleRootDirPrefix.isNotEmpty()) {
            "./$gradleRootDirPrefix/gradlew -p ./$gradleRootDirPrefix"
        } else {
            "./gradlew"
        }
    return "$gradleCommand --quiet $taskName -P$FILTER_INCLUDE_PROPERTY_NAME=\"${'$'}CHANGED_FILES\""
}

private fun generateGitCommand(gradleRootDirPrefix: String): String =
    if (gradleRootDirPrefix.isEmpty()) {
        "git --no-pager diff --name-status --no-color --cached"
    } else {
        "git --no-pager diff --name-status --no-color --cached -- $gradleRootDirPrefix/"
    }

private fun postCheck(shouldUpdateCommit: Boolean): String =
    if (shouldUpdateCommit) {
        """
    echo "${'$'}CHANGED_FILES" | while read -r file; do
        if [ -f ${'$'}file ]; then
            git add ${'$'}file
        fi
    done
    """
    } else {
        ""
    }

internal const val NF = "\$NF"

/** Upstream's hook script, byte for byte. */
internal fun generateGitHook(taskName: String, shouldUpdateCommit: Boolean, gradleRootDirPrefix: String) =
    """
    set +e
    CHANGED_FILES="${'$'}(${generateGitCommand(gradleRootDirPrefix)} | awk '$1 != "D" && $NF ~ /\.kts?$/ { print $NF }')"

    if [ -z "${'$'}CHANGED_FILES" ]; then
        echo "No Kotlin staged files."
        exit 0
    fi;

    echo "Running ktlint over these files:"
    echo "${'$'}CHANGED_FILES"

    diff=.git/unstaged-ktlint-git-hook.diff
    git diff --binary --color=never > ${'$'}diff
    if [ -s ${'$'}diff ]; then
      git apply -R ${'$'}diff
    fi

    ${generateGradleCommand(taskName, gradleRootDirPrefix)}
    gradle_command_exit_code=${'$'}?

    echo "Completed ktlint run."
    ${postCheck(shouldUpdateCommit)}

    if [ -s ${'$'}diff ]; then
      git apply --ignore-whitespace ${'$'}diff
    fi
    rm ${'$'}diff
    unset diff

    echo "Completed ktlint hook."
    exit ${'$'}gradle_command_exit_code

    """
        .trimIndent()

internal fun BaseKtLintCheckTask.applyGitFilter() {
    val projectRelativePath =
        project.rootDir.toPath().relativize(project.projectDir.toPath()).toString().replace("\\", "/")

    val filesToInclude =
        (project.property(FILTER_INCLUDE_PROPERTY_NAME) as String)
            .lines()
            .map { it.replace("\\", "/") }
            .filter { it.startsWith(projectRelativePath) }

    if (filesToInclude.isNotEmpty()) {
        include { fileTreeElement ->
            if (fileTreeElement.isDirectory) {
                true
            } else {
                filesToInclude.any { fileTreeElement.file.absolutePath.replace("\\", "/").endsWith(it) }
            }
        }
    } else {
        exclude("*")
    }
}

/** The git directory and work tree around [dir], as jgit's `RepositoryBuilder.findGitDir`. */
internal fun findGitDir(dir: File): Pair<File, File>? {
    val dotGit = generateSequence(dir.absoluteFile) { it.parentFile }.map { File(it, ".git") }.firstOrNull { it.exists() }
    val workTree = dotGit?.parentFile ?: return null
    if (dotGit.isDirectory) return dotGit to workTree
    val gitDir =
        dotGit.readLines().firstOrNull { it.startsWith("gitdir:") }?.removePrefix("gitdir:")?.trim() ?: return null
    return File(gitDir).let { if (it.isAbsolute) it else File(workTree, gitDir) } to workTree
}
