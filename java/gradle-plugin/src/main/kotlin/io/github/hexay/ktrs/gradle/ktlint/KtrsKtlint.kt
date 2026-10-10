package io.github.hexay.ktrs.gradle.ktlint

import io.github.hexay.ktrs.KtrsExecutable
import java.io.File
import org.gradle.api.GradleException
import org.gradle.api.logging.Logger

/**
 * One `ktrs ktlint` process (the `ktlint` drop-in) over a task's files. A run that loads a rule set
 * or reporter JAR ktrs can't run natively is handed by ktrs to the real ktlint jar, which finds
 * `java` through `JAVA_HOME`: the Gradle JVM's.
 *
 * @param executable the `ktrs.executable` property, else the binary bundled in `io.github.hexay:ktrs`
 */
internal class KtrsKtlint(
    private val executable: String?,
    private val workingDir: File,
    private val tempDir: File,
    private val logger: Logger,
    private val debug: Boolean,
) {

    /**
     * Runs `ktrs ktlint <options> <files>`; returns the exit code: 0, or 1 for lint errors.
     *
     * @param allowNoFiles for options under which ktlint does not lint the working directory's default patterns
     *   when it gets no file arguments
     */
    fun run(options: List<String>, files: Collection<File>, allowNoFiles: Boolean = false): Int {
        require(allowNoFiles || files.isNotEmpty()) { "no files to lint" }
        val command = listOf(binary(), "ktlint") + options + fileArguments(files)
        log("Running ${command.joinToString(" ")}")
        val process =
            ProcessBuilder(command).directory(workingDir).redirectErrorStream(true).also {
                it.environment()["JAVA_HOME"] = System.getProperty("java.home")
            }.start()
        process.outputStream.close()
        val output = process.inputStream.bufferedReader().readText()
        val exitCode = process.waitFor()
        if (output.isNotBlank()) log(output.trimEnd())
        if (exitCode != 0 && exitCode != 1) {
            throw GradleException("ktrs ktlint failed with exit code $exitCode:\n${output.trimEnd()}")
        }
        return exitCode
    }

    private fun binary(): String = executable ?: KtrsExecutable.locate().toString()

    /** The files, through an argfile when the command line could get too long for Windows. */
    private fun fileArguments(files: Collection<File>): List<String> {
        val paths = files.map { it.absolutePath }
        if (paths.sumOf { it.length + 1 } < MAX_INLINE_LENGTH) return paths
        val argfile = File(tempDir, "files.args")
        argfile.parentFile.mkdirs()
        argfile.writeText(paths.joinToString("\n") { "\"${it.replace("\\", "\\\\").replace("\"", "\\\"")}\"" })
        return listOf("@${argfile.absolutePath}")
    }

    private fun log(message: String) {
        if (debug) logger.warn("[KtLint DEBUG] $message") else logger.info(message)
    }

    private companion object {
        // CreateProcess allows 32767 characters; leave room for the binary and the options.
        const val MAX_INLINE_LENGTH = 24_000
    }
}
