package org.jlleitschuh.gradle.ktlint.tasks

import io.github.hexay.ktrs.gradle.ktlint.JsonReport
import io.github.hexay.ktrs.gradle.ktlint.LoadedReporter
import io.github.hexay.ktrs.gradle.ktlint.ReportedError
import java.io.File
import javax.inject.Inject
import org.gradle.api.DefaultTask
import org.gradle.api.GradleException
import org.gradle.api.file.DirectoryProperty
import org.gradle.api.file.ProjectLayout
import org.gradle.api.file.RegularFileProperty
import org.gradle.api.model.ObjectFactory
import org.gradle.api.provider.Property
import org.gradle.api.provider.SetProperty
import org.jlleitschuh.gradle.ktlint.reporter.ReporterType
import org.gradle.api.tasks.CacheableTask
import org.gradle.api.tasks.Input
import org.gradle.api.tasks.InputDirectory
import org.gradle.api.tasks.InputFile
import org.gradle.api.tasks.Internal
import org.gradle.api.tasks.OutputDirectory
import org.gradle.api.tasks.PathSensitive
import org.gradle.api.tasks.PathSensitivity
import org.gradle.api.tasks.TaskAction
import org.jlleitschuh.gradle.ktlint.capitalizeName

/**
 * Generates reports and prints errors into Gradle console: publishes the reports its lint task's
 * `ktrs ktlint` run wrote, as `<reportsName>.<extension>`.
 *
 * This will actually fail the build in case some non-corrected lint issues.
 */
@CacheableTask
public abstract class GenerateReportsTask
@Inject
constructor(projectLayout: ProjectLayout, objectFactory: ObjectFactory) : DefaultTask() {

    @get:PathSensitive(PathSensitivity.RELATIVE)
    @get:InputFile
    internal abstract val discoveredErrors: RegularFileProperty

    @get:PathSensitive(PathSensitivity.RELATIVE)
    @get:InputDirectory
    internal abstract val lintReports: DirectoryProperty

    @get:PathSensitive(PathSensitivity.RELATIVE)
    @get:InputFile
    internal abstract val loadedReporters: RegularFileProperty

    @get:Input internal abstract val reportsName: Property<String>

    @get:Input internal abstract val outputToConsole: Property<Boolean>

    @get:Input internal abstract val ignoreFailures: Property<Boolean>

    @get:Input internal abstract val verbose: Property<Boolean>

    /** As upstream: switching `plain` to `plain_group_by_file` reruns the task even when the reports match. */
    @get:Input internal abstract val enabledReporters: SetProperty<ReporterType>

    @get:Internal internal val projectDirectory: File = projectLayout.projectDirectory.asFile

    init {
        // Workaround for https://github.com/gradle/gradle/issues/2919
        onlyIf { (it as GenerateReportsTask).discoveredErrors.asFile.get().exists() }
    }

    /** Reports output directory. Default is "build/reports/ktlint/${taskName}/". */
    @get:OutputDirectory
    public val reportsOutputDirectory: DirectoryProperty =
        objectFactory.directoryProperty().convention(
            reportsName.flatMap {
                projectLayout.buildDirectory.dir("reports${File.separator}ktlint${File.separator}$it")
            }
        )

    @TaskAction
    public fun generateReports() {
        val outputDir = reportsOutputDirectory.get().asFile
        val reports =
            LoadedReporter.read(loadedReporters.get().asFile).map {
                File(lintReports.get().asFile, "report.${it.fileExtension}")
                    .copyTo(File(outputDir, "${reportsName.get()}.${it.fileExtension}"), overwrite = true)
            }

        // ktlint's CLI reports an unfixable error once per format pass; the engine API, upstream's, once.
        val lintErrors = JsonReport.read(discoveredErrors.get().asFile).distinct()
        if (outputToConsole.get()) {
            lintErrors.forEach { logError(it) }
        }

        if (!ignoreFailures.get() && lintErrors.isNotEmpty()) {
            val reportsPaths = reports.joinToString(separator = "\n") { "- ${it.absolutePath}" }
            throw GradleException("KtLint found code style violations. Please see the following reports:\n$reportsPaths")
        }
    }

    private fun logError(error: ReportedError) {
        val filePath = error.absolutePath(projectDirectory)
        val verboseSuffix = if (verbose.get()) " (${error.rule})" else ""
        logger.warn("$filePath:${error.line}:${error.column} ${error.message}$verboseSuffix")
    }

    internal enum class LintType(val suffix: String) {
        CHECK("Check"),
        FORMAT("Format"),
    }

    internal companion object {
        internal fun generateNameForSourceSets(sourceSetName: String, lintType: LintType): String =
            "ktlint${sourceSetName.capitalizeName()}SourceSet${lintType.suffix}"

        internal fun generateNameForKotlinScripts(lintType: LintType): String = "ktlintKotlinScript${lintType.suffix}"

        const val DESCRIPTION = "Generates reports and prints errors into Gradle console."
    }
}
