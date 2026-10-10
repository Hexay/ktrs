package org.jmailen.gradle.kotlinter.tasks

import java.io.File
import javax.inject.Inject
import org.gradle.api.file.FileTree
import org.gradle.api.file.ProjectLayout
import org.gradle.api.model.ObjectFactory
import org.gradle.api.provider.MapProperty
import org.gradle.api.tasks.CacheableTask
import org.gradle.api.tasks.InputFiles
import org.gradle.api.tasks.OutputFiles
import org.gradle.api.tasks.PathSensitive
import org.gradle.api.tasks.PathSensitivity
import org.gradle.api.tasks.TaskAction
import org.gradle.work.InputChanges
import org.gradle.workers.WorkerExecutionException
import org.gradle.workers.WorkerExecutor
import org.jmailen.gradle.kotlinter.support.LintFailure
import org.jmailen.gradle.kotlinter.tasks.lint.LintWorkerAction

@CacheableTask
public abstract class LintTask
@Inject
constructor(
    private val workerExecutor: WorkerExecutor,
    objectFactory: ObjectFactory,
    private val projectLayout: ProjectLayout,
) : ConfigurableKtLintTask(projectLayout = projectLayout, objectFactory = objectFactory) {

    @get:OutputFiles public val reports: MapProperty<String, File> = objectFactory.mapProperty(default = emptyMap())

    @InputFiles @PathSensitive(PathSensitivity.RELATIVE) override fun getSource(): FileTree = super.getSource()

    @TaskAction
    public fun run(inputChanges: InputChanges) {
        checkKtlintVersion()
        val workQueue = workerExecutor.ktlintWorkQueue()
        workQueue.submit(LintWorkerAction::class.java) { p ->
            p.name.set(name)
            p.files.from(source)
            p.projectDirectory.set(projectLayout.projectDirectory.asFile)
            p.reporters.putAll(reports)
            p.changedEditorConfigFiles.from(getChangedEditorconfigFiles(inputChanges))
            p.ktlintVersion.set(ktlintVersion)
            p.ktlintClasspath.from(ktlintClasspath)
            p.ktrsExecutable.set(ktrsExecutable)
            p.temporaryDir.set(temporaryDir)
        }
        try {
            workQueue.await()
        } catch (e: WorkerExecutionException) {
            if (!(ignoreLintFailures.get() && e.hasRootCause(LintFailure::class.java))) {
                throw e
            }
        }
    }
}
