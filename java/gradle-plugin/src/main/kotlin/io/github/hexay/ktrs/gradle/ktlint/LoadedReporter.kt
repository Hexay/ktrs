package io.github.hexay.ktrs.gradle.ktlint

import java.io.File

/**
 * A reporter as `ktrs ktlint --reporter=<spec>[,artifact=<jar>],output=<file>` takes it; written by
 * `loadKtlintReporters`, one per line, for the lint tasks.
 *
 * @param spec the reporter id, with its options as a query (`plain?group_by_file`)
 */
internal data class LoadedReporter(val spec: String, val fileExtension: String, val artifact: File?) {

    fun cliOption(output: File): String =
        listOfNotNull("--reporter=$spec", artifact?.let { "artifact=${it.absolutePath}" }, "output=${output.absolutePath}")
            .joinToString(",")

    companion object {
        fun write(reporters: List<LoadedReporter>, file: File) {
            file.parentFile.mkdirs()
            file.writeText(
                reporters.joinToString("") { "${it.spec}\t${it.fileExtension}\t${it.artifact?.absolutePath.orEmpty()}\n" }
            )
        }

        fun read(file: File): List<LoadedReporter> =
            file.readLines().filter { it.isNotEmpty() }.map { line ->
                val (spec, extension, artifact) = line.split('\t')
                LoadedReporter(spec, extension, artifact.takeIf { it.isNotEmpty() }?.let(::File))
            }
    }
}
