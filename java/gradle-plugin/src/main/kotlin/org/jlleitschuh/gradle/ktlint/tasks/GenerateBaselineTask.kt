package org.jlleitschuh.gradle.ktlint.tasks

import io.github.hexay.ktrs.gradle.ktlint.KtlintCommand
import io.github.hexay.ktrs.gradle.ktlint.KtrsKtlint
import javax.inject.Inject
import org.gradle.api.DefaultTask
import org.gradle.api.Task
import org.gradle.api.file.ConfigurableFileCollection
import org.gradle.api.file.ProjectLayout
import org.gradle.api.file.RegularFileProperty
import org.gradle.api.provider.MapProperty
import org.gradle.api.provider.Property
import org.gradle.api.specs.Spec
import org.gradle.api.tasks.CacheableTask
import org.gradle.api.tasks.Classpath
import org.gradle.api.tasks.Input
import org.gradle.api.tasks.InputFiles
import org.gradle.api.tasks.Internal
import org.gradle.api.tasks.OutputFile
import org.gradle.api.tasks.PathSensitive
import org.gradle.api.tasks.PathSensitivity
import org.gradle.api.tasks.TaskAction

/**
 * Generates KtLint baseline file: one `ktrs ktlint` run with ktlint's `baseline` reporter over the
 * sources of every check task. If baseline file is already exists - it will be overwritten.
 */
@CacheableTask
public abstract class GenerateBaselineTask @Inject constructor(projectLayout: ProjectLayout) : DefaultTask() {

    @get:Classpath internal abstract val ktLintClasspath: ConfigurableFileCollection

    @get:Classpath internal abstract val baselineReporterClasspath: ConfigurableFileCollection

    @get:Classpath internal abstract val ruleSetsClasspath: ConfigurableFileCollection

    @get:PathSensitive(PathSensitivity.RELATIVE)
    @get:InputFiles
    internal abstract val sources: ConfigurableFileCollection

    @get:Input internal abstract val ktLintVersion: Property<String>

    @get:Input internal abstract val additionalEditorconfig: MapProperty<String, String>

    @get:Input internal abstract val ktrsVersion: Property<String>

    @get:Internal internal abstract val ktrsExecutable: Property<String>

    @get:Internal internal abstract val debug: Property<Boolean>

    @get:OutputFile public abstract val baselineFile: RegularFileProperty

    private val projectDirectory = projectLayout.projectDirectory.asFile

    final override fun onlyIf(spec: Spec<in Task>) {
        super.onlyIf(spec)
    }

    @TaskAction
    public fun generateBaseline() {
        val baseline = baselineFile.get().asFile.apply { if (exists()) delete() else parentFile.mkdirs() }
        val files = sources.files.sorted()
        if (files.isEmpty()) {
            // What the baseline reporter writes for no errors; ktlint without files would lint the working dir.
            baseline.writeText(listOf(XML_DECLARATION, "<baseline version=\"1.0\">", "</baseline>", "").joinToString(System.lineSeparator()))
        } else {
            val ktrs = KtrsKtlint(ktrsExecutable.orNull, projectDirectory, temporaryDir, logger, debug.get())
            KtlintCommand(ktLintVersion.get(), ruleSetsClasspath, additionalEditorconfig.get(), ktrs, temporaryDir)
                .generateBaseline(files, baseline)
        }
        logger.warn("Baseline was successfully generated into: ${baseline.absolutePath}")
    }

    public companion object {
        public const val NAME: String = "ktlintGenerateBaseline"
        public const val DESCRIPTION: String = "Generates KtLint baseline file"
        private const val XML_DECLARATION = "<?xml version=\"1.0\" encoding=\"utf-8\"?>"
    }
}
