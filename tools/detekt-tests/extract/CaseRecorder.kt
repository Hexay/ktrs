package dev.detekt.test

import dev.detekt.api.Config
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import org.jetbrains.kotlin.psi.KtFile
import java.io.File

/**
 * Called from the recording `Rule.lint` (RuleExtensions.kt here): writes one golden case per distinct
 * (test, rule, config, file) into `-Dgolden.out`, holding what the real rule returned from `visitFile`, before the
 * test-only suppression filter. On-disk format: crates/ktrs-detekt/tests/golden/case.rs.
 */
object CaseRecorder {
    private val out = System.getProperty("golden.out")?.let(::File)
    private val root = System.getProperty("golden.root").orEmpty().replace('\\', '/').trimEnd('/')
    private val seen = mutableSetOf<String>()
    private val taken = mutableSetOf<String>()

    fun record(rule: Rule, file: KtFile, findings: List<Finding>) {
        write(rule, file) { dir, name ->
            if (findings.isNotEmpty()) File(dir, "$name.findings").writeText(findings.joinToString("") { it.row() + "\n" })
        }
    }

    fun recordError(rule: Rule, file: KtFile, error: Throwable) {
        write(rule, file) { dir, name ->
            val rootCause = generateSequence(error) { it.cause }.last()
            File(dir, "$name.error").writeText("${error::class.simpleName}: ${error.message.orEmpty().lines().first()} <- ${rootCause::class.simpleName}\n")
        }
    }

    private fun write(rule: Rule, file: KtFile, results: (File, String) -> Unit) {
        val out = out ?: return
        val test = CaseNaming.CURRENT.get() ?: "outside-test"
        val path = file.virtualFilePath.replace('\\', '/').let { if (root.isNotEmpty() && it.startsWith("$root/")) it.removePrefix(root) else it }
        val options = "{\"rule\":${json(rule.ruleName.value)},\"path\":${json(path)},\"config\":${configJson(rule.config)}"
        if (!seen.add("$test\u0000$options\u0000${file.text}")) return
        val dir = File(out, rule.ruleName.value).apply { mkdirs() }
        val name = uniqueName(dir, test)
        val ext = if (file.name.endsWith(".kts")) "kts" else "kt"
        File(dir, "$name.input.$ext").writeText(file.text)
        File(dir, "$name.options.json").writeText("$options,\"test\":${json(CaseNaming.DISPLAY.get() ?: test)}}\n")
        results(dir, name)
    }

    private fun Finding.row(): String {
        val location = entity.location
        return listOf(location.source, location.endSource, location.text, entity.signature.escape(), message.escape()).joinToString("\t")
    }

    private fun String.escape() = replace("\\", "\\\\").replace("\t", "\\t").replace("\n", "\\n").replace("\r", "\\r")

    /** `TestConfig`'s raw pairs, types kept (`TestConfig` does no coercion); `Config.empty` is `{}`. */
    private fun configJson(config: Config): String =
        when {
            config === Config.empty -> "{}"
            config is TestConfig -> json(config.values)
            else -> "{\"unsupported config\":${json(config::class.qualifiedName.orEmpty())}}"
        }

    private fun json(value: Any?): String =
        when (value) {
            null -> "null"
            is String -> buildString {
                append('"')
                for (c in value) {
                    when {
                        c == '"' -> append("\\\"")
                        c == '\\' -> append("\\\\")
                        c == '\n' -> append("\\n")
                        c == '\r' -> append("\\r")
                        c == '\t' -> append("\\t")
                        c < ' ' -> append("\\u%04x".format(c.code))
                        else -> append(c)
                    }
                }
                append('"')
            }
            is Boolean, is Int, is Long -> value.toString()
            is Map<*, *> -> value.entries.joinToString(",", "{", "}") { "${json(it.key.toString())}:${json(it.value)}" }
            is Iterable<*> -> value.joinToString(",", "[", "]") { json(it) }
            is Array<*> -> value.joinToString(",", "[", "]") { json(it) }
            else -> "{\"unsupported value\":${json(value::class.qualifiedName.orEmpty() + " " + value)}}"
        }

    private fun uniqueName(dir: File, test: String): String {
        var name = test
        var n = 1
        while (!taken.add("${dir.name}/$name")) name = "$test-${++n}"
        return name
    }
}
