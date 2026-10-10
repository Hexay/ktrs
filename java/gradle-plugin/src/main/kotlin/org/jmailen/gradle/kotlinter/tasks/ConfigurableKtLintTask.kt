package org.jmailen.gradle.kotlinter.tasks

import io.github.hexay.ktrs.gradle.KtrsBuildService
import io.github.hexay.ktrs.gradle.kotlinter.KotlinterCommand
import org.gradle.api.file.ConfigurableFileCollection
import org.gradle.api.file.FileCollection
import org.gradle.api.file.ProjectLayout
import org.gradle.api.model.ObjectFactory
import org.gradle.api.provider.ListProperty
import org.gradle.api.provider.MapProperty
import org.gradle.api.provider.Property
import org.gradle.api.tasks.Classpath
import org.gradle.api.tasks.Input
import org.gradle.api.tasks.InputFiles
import org.gradle.api.tasks.Internal
import org.gradle.api.tasks.PathSensitive
import org.gradle.api.tasks.PathSensitivity
import org.gradle.api.tasks.SourceTask
import org.gradle.work.DisableCachingByDefault
import org.gradle.work.FileChange
import org.gradle.work.Incremental
import org.gradle.work.InputChanges
import org.gradle.workers.WorkQueue
import org.gradle.workers.WorkerExecutionException
import org.gradle.workers.WorkerExecutor
import org.jmailen.gradle.kotlinter.KotlinterExtension.Companion.DEFAULT_IGNORE_LINT_FAILURES
import org.jmailen.gradle.kotlinter.support.defaultWorkerJvmArgs
import org.jmailen.gradle.kotlinter.support.findApplicableEditorConfigFiles
import org.jmailen.gradle.kotlinter.support.versionProperties

/**
 * kotlinter's task base. The worker actions run in-process (`noIsolation`) and do upstream's work around one
 * `ktrs ktlint` process ([KotlinterCommand]) instead of ktlint's engine in a worker JVM.
 */
@DisableCachingByDefault(because = "Base class for lint and format task implementations")
public abstract class ConfigurableKtLintTask(projectLayout: ProjectLayout, objectFactory: ObjectFactory) : SourceTask() {

    @get:InputFiles
    @get:PathSensitive(PathSensitivity.RELATIVE)
    @get:Incremental
    internal val editorconfigFiles: FileCollection =
        objectFactory.fileCollection().apply { from(projectLayout.findApplicableEditorConfigFiles().toList()) }

    @get:Input
    public open val ignoreLintFailures: Property<Boolean> = objectFactory.property(default = DEFAULT_IGNORE_LINT_FAILURES)

    /** The rule sets of the `ktlint` configuration: ktlint's own artifacts are ktrs. */
    @get:Classpath public val ktlintClasspath: ConfigurableFileCollection = objectFactory.fileCollection()

    /** Arguments of upstream's worker JVM. Kept for build script compatibility: ktrs runs no JVM worker. */
    @get:Internal public val workerJvmArgs: ListProperty<String> = objectFactory.listProperty(default = defaultWorkerJvmArgs())

    /** `kotlinter { ktlintVersion }`, which upstream resolves into [ktlintClasspath]. */
    @get:Input
    internal val ktlintVersion: Property<String> = objectFactory.property(default = versionProperties.ktlintVersion())

    /** The ktrs release the plugin belongs to: its rules are the task's behaviour. */
    @get:Input
    internal val ktrsVersion: Property<String> =
        objectFactory.property(
            default =
                ConfigurableKtLintTask::class.java.getResource("/io/github/hexay/ktrs/gradle/ktrs-version.txt")?.readText()
                    ?: error("Missing ktrs version")
        )

    /** `-Pktrs.executable` / `-Dktrs.executable`: a different binary of the same release, so not an input. */
    @get:Internal
    internal val ktrsExecutable: Property<String> =
        objectFactory.property(String::class.java).apply {
            set(
                project.providers
                    .gradleProperty(KtrsBuildService.EXECUTABLE_PROPERTY)
                    .orElse(project.providers.systemProperty(KtrsBuildService.EXECUTABLE_PROPERTY))
            )
        }

    protected fun getChangedEditorconfigFiles(inputChanges: InputChanges): List<java.io.File> =
        inputChanges.getFileChanges(editorconfigFiles).map(FileChange::getFile)

    protected fun WorkerExecutor.ktlintWorkQueue(): WorkQueue = noIsolation()

    /** Fails the build for a ktlint version ktrs does not run, before any work. */
    protected fun checkKtlintVersion() {
        KotlinterCommand.versionOption(ktlintVersion.get())
    }
}

internal inline fun <reified T : Any> ObjectFactory.property(default: T? = null): Property<T> =
    property(T::class.java).apply { set(default) }

internal inline fun <reified T : Any> ObjectFactory.listProperty(default: Iterable<T> = emptyList()): ListProperty<T> =
    listProperty(T::class.java).apply { set(default) }

internal inline fun <reified K : Any, reified V : Any> ObjectFactory.mapProperty(
    default: Map<K, V> = emptyMap()
): MapProperty<K, V> = mapProperty(K::class.java, V::class.java).apply { set(default) }

public fun WorkerExecutionException.hasRootCause(type: Class<*>): Boolean {
    // Upstream's check, for exceptions serialized across its worker process boundary.
    return this.stackTraceToString().contains(type.canonicalName)
}
