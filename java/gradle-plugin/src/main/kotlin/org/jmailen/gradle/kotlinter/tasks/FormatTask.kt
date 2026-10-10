package org.jmailen.gradle.kotlinter.tasks

import javax.inject.Inject
import org.gradle.api.file.ProjectLayout
import org.gradle.api.file.RegularFileProperty
import org.gradle.api.model.ObjectFactory
import org.gradle.api.provider.Property
import org.gradle.api.tasks.Input
import org.gradle.api.tasks.Optional
import org.gradle.api.tasks.OutputFile
import org.gradle.api.tasks.TaskAction
import org.gradle.work.DisableCachingByDefault
import org.gradle.work.InputChanges
import org.gradle.workers.WorkerExecutionException
import org.gradle.workers.WorkerExecutor
import org.jmailen.gradle.kotlinter.KotlinterExtension
import org.jmailen.gradle.kotlinter.support.LintFailure
import org.jmailen.gradle.kotlinter.tasks.format.FormatWorkerAction

@DisableCachingByDefault(because = "Formats source files in place")
public abstract class FormatTask
@Inject
constructor(
    private val workerExecutor: WorkerExecutor,
    objectFactory: ObjectFactory,
    private val projectLayout: ProjectLayout,
) : ConfigurableKtLintTask(projectLayout = projectLayout, objectFactory = objectFactory) {

    @get:OutputFile @get:Optional public val report: RegularFileProperty = objectFactory.fileProperty()

    @get:Input
    public val ignoreFormatFailures: Property<Boolean> =
        objectFactory.property(default = KotlinterExtension.DEFAULT_IGNORE_FORMAT_FAILURES)

    init {
        outputs.upToDateWhen { false }
    }

    @TaskAction
    public fun run(inputChanges: InputChanges) {
        checkKtlintVersion()
        val workQueue = workerExecutor.ktlintWorkQueue()
        workQueue.submit(FormatWorkerAction::class.java) { p ->
            p.name.set(name)
            p.files.from(source)
            p.projectDirectory.set(projectLayout.projectDirectory.asFile)
            p.output.set(report)
            p.changedEditorConfigFiles.from(getChangedEditorconfigFiles(inputChanges))
            p.ktlintVersion.set(ktlintVersion)
            p.ktlintClasspath.from(ktlintClasspath)
            p.ktrsExecutable.set(ktrsExecutable)
            p.temporaryDir.set(temporaryDir)
        }
        try {
            workQueue.await()
        } catch (e: WorkerExecutionException) {
            if (e.hasRootCause(LintFailure::class.java)) {
                if (!ignoreFormatFailures.get()) {
                    throw e
                }
            } else {
                throw e
            }
        }
    }
}
