package io.github.ktlint.core.test

import org.junit.jupiter.api.extension.AfterEachCallback
import org.junit.jupiter.api.extension.BeforeEachCallback
import org.junit.jupiter.api.extension.ExtensionContext
import org.junit.jupiter.api.extension.TestWatcher
import java.io.File

/**
 * Autodetected JUnit extension (META-INF/services): names the running test for [CaseRecorder] and lists upstream
 * failures in `<golden.out>/upstream-failures.txt`. Needs sequential execution (JUnit's default).
 */
public class CaseNaming :
    BeforeEachCallback,
    AfterEachCallback,
    TestWatcher {
    override fun beforeEach(context: ExtensionContext) {
        CURRENT.set(caseName(context))
        DISPLAY.set(display(context))
    }

    override fun afterEach(context: ExtensionContext) {
        CURRENT.remove()
        DISPLAY.remove()
    }

    override fun testFailed(
        context: ExtensionContext,
        cause: Throwable?,
    ) {
        val out = System.getProperty("golden.out") ?: return
        File(out).mkdirs()
        File(out, "upstream-failures.txt").appendText("${display(context)}\t${cause?.message.orEmpty().lines().first()}\n")
    }

    public companion object {
        public val CURRENT: ThreadLocal<String> = ThreadLocal()
        public val DISPLAY: ThreadLocal<String> = ThreadLocal()

        /** `TestClass.Nested.method[i]`: the class chain, the method name, the invocation indices. */
        private fun display(context: ExtensionContext): String {
            val classes = generateSequence(context.requiredTestClass) { it.enclosingClass }.toList().reversed().map { it.simpleName }
            val invocations = Regex("#(\\d+)").findAll(context.uniqueId).map { it.groupValues[1] }.toList()
            return (classes + context.requiredTestMethod.name).joinToString(".") + invocations.joinToString("") { "[$it]" }
        }

        /**
         * `<innermost nested class, 20 chars>--<method, 40 chars>[-p<i>]`, lower-cased (case-insensitive file systems) and
         * short so paths stay under Windows' MAX_PATH. The full name is the `test=` line of `.options`.
         */
        private fun caseName(context: ExtensionContext): String {
            val nested = generateSequence(context.requiredTestClass) { it.enclosingClass }.toList().reversed().drop(1)
            val invocations = Regex("#(\\d+)").findAll(context.uniqueId).map { "p" + it.groupValues[1] }.toList()
            val parts =
                listOfNotNull(
                    nested.lastOrNull()?.let { sanitize(it.simpleName).take(20).trimEnd('-') },
                    sanitize(context.requiredTestMethod.name).take(40).trimEnd('-'),
                )
            return (listOf(parts.joinToString("--")) + invocations).joinToString("-")
        }

        private fun sanitize(s: String) = s.lowercase().replace(Regex("[^a-z0-9]+"), "-").trim('-')
    }
}
