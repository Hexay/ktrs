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

        /** Lower-cased (checkouts on case-insensitive file systems) and capped, like ktfmt's golden names. */
        private fun caseName(context: ExtensionContext): String {
            val nested = generateSequence(context.requiredTestClass) { it.enclosingClass }.toList().reversed().drop(1).map { it.simpleName }
            val invocations = Regex("#(\\d+)").findAll(context.uniqueId).map { "p" + it.groupValues[1] }.toList()
            val base =
                (nested + context.requiredTestMethod.name)
                    .joinToString("--") { sanitize(it) }
                    .take(110)
                    .trimEnd('-')
            return (listOf(base) + invocations).joinToString("-")
        }

        private fun sanitize(s: String) = s.lowercase().replace(Regex("[^a-z0-9]+"), "-").trim('-')
    }
}
