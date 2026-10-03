package org.jlleitschuh.gradle.ktlint.tasks

import io.github.hexay.ktrs.gradle.ktlint.JarServices
import io.github.hexay.ktrs.gradle.ktlint.KtlintVersions
import io.github.hexay.ktrs.gradle.ktlint.LoadedReporter
import javax.inject.Inject
import org.gradle.api.DefaultTask
import org.gradle.api.GradleException
import org.gradle.api.file.ConfigurableFileCollection
import org.gradle.api.file.ProjectLayout
import org.gradle.api.file.RegularFileProperty
import org.gradle.api.model.ObjectFactory
import org.gradle.api.provider.Property
import org.gradle.api.provider.SetProperty
import org.gradle.api.tasks.CacheableTask
import org.gradle.api.tasks.Classpath
import org.gradle.api.tasks.Input
import org.gradle.api.tasks.OutputFile
import org.gradle.api.tasks.TaskAction
import org.jlleitschuh.gradle.ktlint.intermediateResultsBuildDir
import org.jlleitschuh.gradle.ktlint.reporter.CustomReporter
import org.jlleitschuh.gradle.ktlint.reporter.ReporterType

/**
 * Resolves the enabled reporters into `ktrs ktlint --reporter` specs (custom ones with the
 * `ktlintReporter` JAR that provides them), and checks the ktlint version.
 */
@CacheableTask
internal abstract class LoadReportersTask
@Inject
constructor(objectFactory: ObjectFactory, projectLayout: ProjectLayout) : DefaultTask() {

    @get:Classpath internal abstract val ktLintClasspath: ConfigurableFileCollection

    @get:Classpath internal abstract val reportersClasspath: ConfigurableFileCollection

    @get:Input internal abstract val debug: Property<Boolean>

    @get:Input internal abstract val ktLintVersion: Property<String>

    @get:Input internal abstract val enabledReporters: SetProperty<ReporterType>

    @get:Input internal abstract val customReporters: SetProperty<CustomReporter>

    @get:OutputFile
    internal val loadedReporters: RegularFileProperty =
        objectFactory.fileProperty().convention(projectLayout.intermediateResultsBuildDir("reporters.tsv"))

    @TaskAction
    fun loadReporters() {
        val version = ktLintVersion.get()
        KtlintVersions.cliOption(version)
        val builtIn =
            enabledReporters.get().ifEmpty { setOf(ReporterType.PLAIN) }.map { type ->
                val query = if (type.options.isEmpty()) "" else type.options.joinToString("&", prefix = "?")
                LoadedReporter(type.reporterName + query, type.fileExtension, null)
            }
        val custom = customReporters.get().toList()
        val jars = if (custom.isEmpty()) emptyList() else JarServices.reporterJars(reportersClasspath, version)
        val loaded =
            custom.mapIndexed { index, reporter ->
                if (jars.isEmpty()) {
                    throw GradleException("KtLint plugin failed to load ${reporter.reporterId} custom reporter.")
                }
                // ktlint loads the reporters of every `artifact=` JAR together, so any declaring JAR works.
                LoadedReporter(reporter.reporterId, reporter.fileExtension, jars[minOf(index, jars.lastIndex)])
            }
        // One report file per extension: `plain` and `plain_group_by_file` share `.txt`.
        val reporters = (builtIn + loaded).associateBy { it.fileExtension }.values.toList()
        LoadedReporter.write(reporters, loadedReporters.get().asFile)
    }

    internal companion object {
        internal const val TASK_NAME = "loadKtlintReporters"
        internal const val DESCRIPTION = "Preloads required KtLint reporters."
    }
}
