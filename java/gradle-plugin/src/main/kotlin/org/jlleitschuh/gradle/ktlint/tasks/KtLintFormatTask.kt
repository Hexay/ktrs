package org.jlleitschuh.gradle.ktlint.tasks

import io.github.hexay.ktrs.gradle.ktlint.FormatTaskSnapshot
import io.github.hexay.ktrs.gradle.ktlint.FormatTaskSnapshot.Companion.contentHash
import javax.inject.Inject
import org.gradle.api.file.ProjectLayout
import org.gradle.api.file.RegularFileProperty
import org.gradle.api.model.ObjectFactory
import org.gradle.api.tasks.CacheableTask
import org.gradle.api.tasks.LocalState
import org.gradle.api.tasks.TaskAction
import org.gradle.api.tasks.util.PatternFilterable
import org.jlleitschuh.gradle.ktlint.capitalizeName
import org.jlleitschuh.gradle.ktlint.intermediateResultsBuildDir

@CacheableTask
public abstract class KtLintFormatTask
@Inject
constructor(objectFactory: ObjectFactory, projectLayout: ProjectLayout, patternFilterable: PatternFilterable) :
    BaseKtLintCheckTask(objectFactory, projectLayout, patternFilterable) {

    @get:LocalState
    internal val previousRunSnapshot: RegularFileProperty =
        objectFactory.fileProperty().convention(projectLayout.intermediateResultsBuildDir("$name-snapshot.bin"))

    init {
        // Not UP-TO-DATE when a file formatted last time is back to its pre-format content.
        outputs.upToDateWhen {
            val inputSources = source.files
            FormatTaskSnapshot.readFromFile(previousRunSnapshot.get().asFile).formattedSources.none {
                inputSources.contains(it.key) && contentHash(it.key).contentEquals(it.value)
            }
        }
    }

    @TaskAction
    public fun format() {
        val before = source.files.associateWith { contentHash(it) }
        runKtlint(format = true)
        val formatted = before.filter { (file, hash) -> file.exists() && !contentHash(file).contentEquals(hash) }
        if (formatted.isNotEmpty()) {
            FormatTaskSnapshot.writeIntoFile(previousRunSnapshot.get().asFile, FormatTaskSnapshot(formatted))
        }
    }

    internal companion object {
        fun buildTaskNameForSourceSet(sourceSetName: String): String =
            "runKtlintFormatOver${sourceSetName.capitalizeName()}SourceSet"

        const val KOTLIN_SCRIPT_TASK_NAME = "runKtlintFormatOverKotlinScripts"

        fun buildDescription(fileType: String): String =
            "Lints all $fileType files to ensure that they are formatted according to the code style " +
                " and, on error, tries to format code to conform code style."
    }
}
