package com.ncorti.ktfmt.gradle.tasks

import com.ncorti.ktfmt.gradle.FormattingOptionsBean
import com.ncorti.ktfmt.gradle.util.KtfmtResultSummary
import com.ncorti.ktfmt.gradle.util.d
import io.github.hexay.ktrs.gradle.KtfmtFormatResult
import io.github.hexay.ktrs.gradle.KtrsBuildService
import io.github.hexay.ktrs.gradle.SourceFileFormatter
import java.io.File
import org.gradle.api.file.ConfigurableFileCollection
import org.gradle.api.file.FileTree
import org.gradle.api.file.ProjectLayout
import org.gradle.api.file.RegularFile
import org.gradle.api.provider.ListProperty
import org.gradle.api.provider.Property
import org.gradle.api.provider.Provider
import org.gradle.api.tasks.Classpath
import org.gradle.api.tasks.IgnoreEmptyDirectories
import org.gradle.api.tasks.Input
import org.gradle.api.tasks.InputFiles
import org.gradle.api.tasks.Internal
import org.gradle.api.tasks.PathSensitive
import org.gradle.api.tasks.PathSensitivity
import org.gradle.api.tasks.SkipWhenEmpty
import org.gradle.api.tasks.SourceTask
import org.gradle.api.tasks.TaskAction
import org.gradle.api.tasks.options.Option
import org.gradle.work.DisableCachingByDefault

/** ktfmt-gradle's base task: formats every source file through the build's [KtrsBuildService]. */
@DisableCachingByDefault(because = "Subclasses define their own caching strategy")
@Suppress("LeakingThis")
public abstract class KtfmtBaseTask(private val layout: ProjectLayout) : SourceTask() {

    init {
        includeOnly.convention("")
    }

    /** Accepted for compatibility and ignored (ktfmt-gradle ran ktfmt from this classpath). */
    @get:Classpath @get:InputFiles public abstract val ktfmtClasspath: ConfigurableFileCollection

    @get:Input public abstract val formattingOptionsBean: Property<FormattingOptionsBean>

    @get:Option(
        option = "include-only",
        description =
            "A comma separate list of relative file paths to include exclusively. " +
                "If set the task will run the processing only on such files.",
    )
    @get:Input
    public abstract val includeOnly: Property<String>

    /** Accepted for compatibility and ignored. */
    @get:Input public abstract val useClassloaderIsolation: Property<Boolean>

    /** Accepted for compatibility and ignored. */
    @get:Input public abstract val processIsolationJvmArgs: ListProperty<String>

    /** The formatter's version: replaces ktfmt-gradle's `ktfmtClasspath` in the cache key. */
    @get:Input public abstract val ktrsVersion: Property<String>

    @get:Internal public abstract val ktrsService: Property<KtrsBuildService>

    @PathSensitive(PathSensitivity.RELATIVE)
    @InputFiles
    @IgnoreEmptyDirectories
    @SkipWhenEmpty
    override fun getSource(): FileTree = super.getSource()

    @get:Internal
    protected val defaultOutput: Provider<RegularFile>
        get() = layout.buildDirectory.file("ktfmt/${this.name}/output.txt")

    @get:Internal public abstract val output: Provider<RegularFile>

    @get:Internal public abstract val reformatFiles: Boolean

    /**
     * Called after all files have been analyzed and [resultSummary] has been written into [output].
     */
    protected abstract fun handleResultSummary(resultSummary: KtfmtResultSummary)

    @TaskAction
    internal fun taskAction() {
        val service = ktrsService.get()
        val options = formattingOptionsBean.get()
        if (options.debuggingPrintOpsAfterFormatting) {
            service.warnDebuggingOpsUnsupported(logger)
        }
        val formatter =
            SourceFileFormatter(service, options, getIncludedFiles(), reformatFiles, logger)
        val results = collectResults(service.mapInParallel(source.files.toList(), formatter::process))

        reportFailedFiles(results)
        writeResultsSummaryToOutput(results)
        handleResultSummary(results)
    }

    private fun getIncludedFiles(): Set<File> {
        val includedFiles =
            IncludedFilesParser.parse(includeOnly.get(), layout.projectDirectory.asFile)
        logger.d(
            "Preparing to format: includeOnly=${includeOnly.orNull}, includedFiles = $includedFiles"
        )
        return includedFiles
    }

    private fun collectResults(results: List<KtfmtFormatResult>): KtfmtResultSummary {
        fun files(predicate: (KtfmtFormatResult) -> Boolean) =
            results.filter(predicate).map { it.input }

        return KtfmtResultSummary(
            files { it is KtfmtFormatResult.KtfmtFormatSuccess && it.wasCorrectlyFormatted },
            files { it is KtfmtFormatResult.KtfmtFormatSuccess && !it.wasCorrectlyFormatted },
            files { it is KtfmtFormatResult.KtfmtFormatSkipped },
            files { it is KtfmtFormatResult.KtfmtFormatFailure },
        )
    }

    private fun reportFailedFiles(resultSummary: KtfmtResultSummary) {
        if (resultSummary.failedFiles.isNotEmpty()) {
            val fileList =
                resultSummary.failedFiles.joinToString("\n") {
                    it.relativeTo(layout.projectDirectory.asFile).path
                }

            error("Ktfmt failed to run with ${resultSummary.failedFiles.size} failures:\n$fileList")
        }
    }

    private fun writeResultsSummaryToOutput(results: KtfmtResultSummary) =
        output.get().asFile.apply { parentFile.mkdirs() }.writeText(results.prettyPrint())
}
