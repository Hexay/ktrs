package org.jlleitschuh.gradle.ktlint.tasks

import javax.inject.Inject
import org.gradle.api.file.ProjectLayout
import org.gradle.api.model.ObjectFactory
import org.gradle.api.tasks.CacheableTask
import org.gradle.api.tasks.TaskAction
import org.gradle.api.tasks.util.PatternFilterable
import org.gradle.work.InputChanges
import org.jlleitschuh.gradle.ktlint.capitalizeName

@CacheableTask
public abstract class KtLintCheckTask
@Inject
constructor(objectFactory: ObjectFactory, projectLayout: ProjectLayout, patternFilterable: PatternFilterable) :
    BaseKtLintCheckTask(objectFactory, projectLayout, patternFilterable) {

    @TaskAction
    public fun lint(inputChanges: InputChanges) {
        runLint(inputChanges)
    }

    internal companion object {
        fun buildTaskNameForSourceSet(sourceSetName: String): String =
            "runKtlintCheckOver${sourceSetName.capitalizeName()}SourceSet"

        const val KOTLIN_SCRIPT_TASK_NAME = "runKtlintCheckOverKotlinScripts"

        fun buildDescription(fileType: String): String =
            "Lints all $fileType files to ensure that they are formatted according to the code style."
    }
}
