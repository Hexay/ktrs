package io.github.hexay.ktrs.gradle

import com.ncorti.ktfmt.gradle.util.w
import io.github.hexay.ktrs.Ktrs
import io.github.hexay.ktrs.KtrsOptions
import java.io.File
import java.nio.file.Paths
import java.util.concurrent.Callable
import java.util.concurrent.ExecutionException
import java.util.concurrent.ExecutorService
import java.util.concurrent.Executors
import java.util.concurrent.atomic.AtomicBoolean
import org.gradle.api.logging.Logger
import org.gradle.api.provider.Property
import org.gradle.api.services.BuildService
import org.gradle.api.services.BuildServiceParameters

/**
 * The build's formatter: one pool of `ktrs serve` processes and one thread per core feeding them,
 * shared by every ktfmt task and closed when the build finishes.
 *
 * The binary is the one bundled in `io.github.hexay:ktrs`. The Gradle property `ktrs.executable`
 * (`-Pktrs.executable=<path>` or `gradle.properties`) or the `ktrs.executable` system property
 * points it at another one instead.
 */
public abstract class KtrsBuildService : BuildService<KtrsBuildService.Params>, AutoCloseable {

    public interface Params : BuildServiceParameters {
        public val executable: Property<String>
    }

    private val ktrsInstance = lazy {
        parameters.executable.orNull?.let { Ktrs.create(Paths.get(it)) } ?: Ktrs.create()
    }
    private val executorInstance = lazy {
        Executors.newFixedThreadPool(Runtime.getRuntime().availableProcessors()) { runnable ->
            Thread(runnable, "ktrs-format").apply { isDaemon = true }
        }
    }
    private val ktrs: Ktrs by ktrsInstance
    private val executor: ExecutorService by executorInstance
    private val warnedAboutDebuggingOps = AtomicBoolean()

    internal fun format(code: String, options: KtrsOptions, file: File): String =
        ktrs.format(code, options, file.toPath())

    /** [action] over [items] on the shared threads, results in [items]' order. */
    internal fun <T, R> mapInParallel(items: List<T>, action: (T) -> R): List<R> =
        items
            .map { item -> executor.submit(Callable { action(item) }) }
            .map { future ->
                try {
                    future.get()
                } catch (e: ExecutionException) {
                    throw e.cause ?: e
                }
            }

    internal fun warnDebuggingOpsUnsupported(logger: Logger) {
        if (warnedAboutDebuggingOps.compareAndSet(false, true)) {
            logger.w("debuggingPrintOpsAfterFormatting is not supported by ktrs and is ignored")
        }
    }

    override fun close() {
        if (executorInstance.isInitialized()) executor.shutdownNow()
        if (ktrsInstance.isInitialized()) ktrs.close()
    }

    internal companion object {
        const val NAME = "ktrs"
        const val EXECUTABLE_PROPERTY = "ktrs.executable"
    }
}
