package org.jlleitschuh.gradle.ktlint.tasks

import groovy.lang.Closure
import io.github.hexay.ktrs.gradle.ktlint.KtlintCommand
import java.io.File
import javax.inject.Inject
import org.gradle.api.DefaultTask
import org.gradle.api.file.ConfigurableFileCollection
import org.gradle.api.file.DirectoryProperty
import org.gradle.api.file.FileCollection
import org.gradle.api.file.FileTreeElement
import org.gradle.api.file.ProjectLayout
import org.gradle.api.file.RegularFileProperty
import org.gradle.api.model.ObjectFactory
import org.gradle.api.provider.MapProperty
import org.gradle.api.provider.Property
import org.gradle.api.specs.Spec
import org.gradle.api.tasks.CacheableTask
import org.gradle.api.tasks.Classpath
import org.gradle.api.tasks.IgnoreEmptyDirectories
import org.gradle.api.tasks.Input
import org.gradle.api.tasks.InputFile
import org.gradle.api.tasks.InputFiles
import org.gradle.api.tasks.Internal
import org.gradle.api.tasks.Optional
import org.gradle.api.tasks.OutputDirectory
import org.gradle.api.tasks.OutputFile
import org.gradle.api.tasks.PathSensitive
import org.gradle.api.tasks.PathSensitivity
import org.gradle.api.tasks.SkipWhenEmpty
import org.gradle.api.tasks.util.PatternFilterable
import org.jlleitschuh.gradle.ktlint.FILTER_INCLUDE_PROPERTY_NAME
import org.jlleitschuh.gradle.ktlint.KOTLIN_EXTENSIONS
import org.jlleitschuh.gradle.ktlint.applyGitFilter
import org.jlleitschuh.gradle.ktlint.getEditorConfigFiles
import org.jlleitschuh.gradle.ktlint.intermediateResultsBuildDir

/**
 * ktlint-gradle's lint/format task base: one `ktrs ktlint` run over all of [source], writing the
 * reports (for [GenerateReportsTask] to publish) and the errors for the console ([discoveredErrors],
 * ktlint's `json` report).
 */
@CacheableTask
public abstract class BaseKtLintCheckTask
@Inject
constructor(
    private val objectFactory: ObjectFactory,
    projectLayout: ProjectLayout,
    private val patternFilterable: PatternFilterable,
) : DefaultTask(), PatternFilterable {

    @get:Classpath internal abstract val ktLintClasspath: ConfigurableFileCollection

    @get:Input internal abstract val additionalEditorconfig: MapProperty<String, String>

    @get:PathSensitive(PathSensitivity.RELATIVE)
    @get:InputFiles
    internal val editorConfigFiles: FileCollection =
        objectFactory.fileCollection().from({ getEditorConfigFiles(projectLayout.projectDirectory.asFile.toPath()) })

    @get:Input internal abstract val ktLintVersion: Property<String>

    @get:Classpath internal abstract val ruleSetsClasspath: ConfigurableFileCollection

    @get:Input internal abstract val debug: Property<Boolean>

    @get:Input internal abstract val android: Property<Boolean>

    @get:Input internal abstract val enableExperimentalRules: Property<Boolean>

    @get:Input internal abstract val relative: Property<Boolean>

    @get:Input internal abstract val coloredOutput: Property<Boolean>

    @get:Input internal abstract val outputColorName: Property<String>

    @get:PathSensitive(PathSensitivity.RELATIVE)
    @get:InputFile
    @get:Optional
    internal abstract val baseline: RegularFileProperty

    /** The ktrs release the plugin belongs to: its rules are the task's behaviour. */
    @get:Input internal abstract val ktrsVersion: Property<String>

    /** `-Pktrs.executable`: a different binary of the same release, so not an input. */
    @get:Internal internal abstract val ktrsExecutable: Property<String>

    /** Max lint worker heap size. Kept for build script compatibility: ktrs runs no JVM worker. */
    @get:Internal
    public val workerMaxHeapSize: Property<String> = objectFactory.property(String::class.java).convention("256m")

    private var sourceFiles: ConfigurableFileCollection = objectFactory.fileCollection()

    @get:Internal internal val projectDirectory: File = projectLayout.projectDirectory.asFile

    init {
        if (project.providers.gradleProperty(FILTER_INCLUDE_PROPERTY_NAME).orNull != null) {
            applyGitFilter()
        } else {
            KOTLIN_EXTENSIONS.forEach { include("**/*.$it") }
        }
    }

    @get:IgnoreEmptyDirectories
    @get:SkipWhenEmpty
    @get:PathSensitive(PathSensitivity.RELATIVE)
    @get:InputFiles
    public val source: FileCollection =
        objectFactory.fileCollection().from({ sourceFiles.asFileTree.matching(patternFilterable) })

    /** Sets the source from this task, evaluated as per [org.gradle.api.Project.file]. */
    public fun setSource(source: Any): BaseKtLintCheckTask {
        sourceFiles = objectFactory.fileCollection().from(source)
        return this
    }

    /** Adds some source to this task, evaluated as per [org.gradle.api.Project.files]. */
    public fun source(vararg sources: Any): BaseKtLintCheckTask {
        sourceFiles.from(sources)
        return this
    }

    @get:PathSensitive(PathSensitivity.RELATIVE)
    @get:InputFile
    internal abstract val loadedReporters: RegularFileProperty

    @get:OutputFile
    internal val discoveredErrors: RegularFileProperty =
        objectFactory.fileProperty().convention(projectLayout.intermediateResultsBuildDir("${name}_errors.json"))

    /** The reports of the run, as `report.<extension>`. */
    @get:OutputDirectory
    internal val reportsDirectory: DirectoryProperty =
        objectFactory.directoryProperty().convention(
            projectLayout.buildDirectory.dir("intermediates/ktLint/$name-reports")
        )

    @Internal override fun getIncludes(): MutableSet<String> = patternFilterable.includes

    @Internal override fun getExcludes(): MutableSet<String> = patternFilterable.excludes

    override fun setIncludes(includes: MutableIterable<String>): BaseKtLintCheckTask =
        also { patternFilterable.setIncludes(includes) }

    override fun setExcludes(excludes: MutableIterable<String>): BaseKtLintCheckTask =
        also { patternFilterable.setExcludes(excludes) }

    override fun include(vararg includes: String): BaseKtLintCheckTask = also { patternFilterable.include(*includes) }

    override fun include(includes: MutableIterable<String>): BaseKtLintCheckTask =
        also { patternFilterable.include(includes) }

    override fun include(includeSpec: Spec<FileTreeElement>): BaseKtLintCheckTask =
        also { patternFilterable.include(includeSpec) }

    override fun include(includeSpec: Closure<*>): BaseKtLintCheckTask = also { patternFilterable.include(includeSpec) }

    override fun exclude(vararg excludes: String): BaseKtLintCheckTask = also { patternFilterable.exclude(*excludes) }

    override fun exclude(excludes: MutableIterable<String>): BaseKtLintCheckTask =
        also { patternFilterable.exclude(excludes) }

    override fun exclude(excludeSpec: Spec<FileTreeElement>): BaseKtLintCheckTask =
        also { patternFilterable.exclude(excludeSpec) }

    override fun exclude(excludeSpec: Closure<*>): BaseKtLintCheckTask = also { patternFilterable.exclude(excludeSpec) }

    /** Lints (or with [format], formats) every source file in one `ktrs ktlint` run. */
    protected fun runKtlint(format: Boolean) {
        val files = source.files.sorted()
        logger.debug("Linting files: ${files.joinToString()}")
        val reportsDir = reportsDirectory.get().asFile
        reportsDir.deleteRecursively()
        reportsDir.mkdirs()
        val errors = discoveredErrors.get().asFile
        errors.delete()
        KtlintCommand(this).lint(this, files, format, reportsDir, errors)
    }
}
