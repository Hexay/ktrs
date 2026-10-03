package org.jlleitschuh.gradle.ktlint

import java.io.File
import org.jlleitschuh.gradle.ktlint.tasks.GenerateReportsTask
import org.junit.jupiter.api.Assumptions
import org.junit.jupiter.api.io.TempDir

abstract class AbstractPluginTest {

    @TempDir lateinit var temporaryFolder: File

    val projectRoot: File
        get() = temporaryFolder.resolve("plugin-test").apply { mkdirs() }

    val mainSourceSetCheckTaskName =
        GenerateReportsTask.generateNameForSourceSets("main", GenerateReportsTask.LintType.CHECK)

    val mainSourceSetFormatTaskName =
        GenerateReportsTask.generateNameForSourceSets("main", GenerateReportsTask.LintType.FORMAT)

    val kotlinScriptCheckTaskName = GenerateReportsTask.generateNameForKotlinScripts(GenerateReportsTask.LintType.CHECK)

    protected fun File.withCleanSources() = createSourceFile("src/main/kotlin/CleanSource.kt", "val foo = \"bar\"\n")

    protected fun File.withCleanKotlinScript() = createSourceFile("kotlin-script.kts", "println(\"zzz\")\n")

    protected fun File.withFailingKotlinScript() = createSourceFile("kotlin-script-fail.kts", "println(\"zzz\") \n")

    protected fun File.withAlternativeFailingSources(baseDir: String) =
        createSourceFile("$baseDir/FailSource.kt", """val  foo    =     "bar"""")

    protected fun File.createSourceFile(sourceFilePath: String, contents: String) {
        val sourceFile = resolve(sourceFilePath)
        sourceFile.parentFile.mkdirs()
        sourceFile.writeText(contents)
    }
}

fun File.buildFile(): File = resolve("build.gradle")

/** `git init` in this directory; returns the `.git` directory. Skips the test without git. */
internal fun File.initGit(): File {
    val exitCode =
        try {
            ProcessBuilder("git", "init", "-q", absolutePath).redirectErrorStream(true).start().run {
                inputStream.readBytes()
                waitFor()
            }
        } catch (_: java.io.IOException) {
            -1
        }
    Assumptions.assumeTrue(exitCode == 0, "git is not available")
    return resolve(".git")
}

internal fun File.initGitWithoutHooksDir(): File {
    val gitDir = initGit()
    check(gitDir.resolve("hooks").deleteRecursively())
    return gitDir
}
