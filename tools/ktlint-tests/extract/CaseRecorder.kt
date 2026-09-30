package io.github.ktlint.core.test

import io.github.ktlint.core.rule.engine.api.Code
import io.github.ktlint.core.rule.engine.api.EditorConfigOverride
import io.github.ktlint.core.rule.engine.api.KtLintRuleEngine
import io.github.ktlint.core.rule.engine.api.LintError
import io.github.ktlint.core.rule.engine.core.api.AutocorrectDecision
import io.github.ktlint.core.rule.engine.core.api.RuleV2Provider
import java.io.File
import java.nio.file.FileSystem

/**
 * Called from the patched `KtLintAssertThatAssertable.init` (see extract-goldens.sh): writes one golden case per distinct
 * (rules, path, editorconfig, code) of the running test into `-Dgolden.out`, with the expectations computed by the real
 * engine, configured exactly as `createKtLintRuleEngine()` does. On-disk format: research/12 §3 and tests/golden.rs.
 */
public object CaseRecorder {
    private val out = System.getProperty("golden.out")?.let(::File)
    private val seen = mutableSetOf<String>()
    private val taken = mutableSetOf<String>()

    @JvmStatic
    public fun record(
        ruleProvider: RuleV2Provider,
        additionalRuleProviders: Set<RuleV2Provider>,
        code: Code,
        editorConfigOverride: EditorConfigOverride,
        fileSystem: FileSystem,
    ) {
        val out = out ?: return
        val test = CaseNaming.CURRENT.get() ?: "outside-test"
        val providers = linkedSetOf(ruleProvider) + additionalRuleProviders
        val options =
            buildString {
                append("rules=").append(providers.joinToString(",") { it.ruleId.value }).append('\n')
                code.filePath?.let { append("path=").append(it.toString().replace('\\', '/')).append('\n') }
                for ((property, value) in editorConfigOverride.properties) {
                    append("ec.").append(property.name).append('=').append(value.source).append('\n')
                }
            }
        if (!seen.add("$test\u0000$options\u0000${code.script}\u0000${code.content}")) return
        val dir = File(out, ruleDir(ruleProvider.ruleId.value)).apply { mkdirs() }
        val name = uniqueName(dir, test)
        val ext = if (code.script) "kts" else "kt"
        File(dir, "$name.input.$ext").writeText(code.content)
        File(dir, "$name.options").writeText(options + "test=${CaseNaming.DISPLAY.get()}\n")

        val engine = KtLintRuleEngine(ruleProviders = providers, editorConfigOverride = editorConfigOverride, fileSystem = fileSystem)
        val errors = StringBuilder()
        val lint = StringBuilder()
        runCatching { engine.lint(code) { lint.append(it.row()).append('\n') } }
            .onFailure { errors.append("lint\t").append(it.describe()).append('\n') }
        val format = StringBuilder()
        runCatching {
            engine.format(code) { e ->
                format.append(e.row()).append('\n')
                AutocorrectDecision.ALLOW_AUTOCORRECT
            }
        }.onSuccess { formatted -> if (formatted != code.content) File(dir, "$name.expected.$ext").writeText(formatted) }
            .onFailure { errors.append("format\t").append(it.describe()).append('\n') }
        if (lint.isNotEmpty()) File(dir, "$name.lint").writeText(lint.toString())
        if (format.isNotEmpty()) File(dir, "$name.format").writeText(format.toString())
        if (errors.isNotEmpty()) File(dir, "$name.error").writeText(errors.toString())
    }

    private fun LintError.row() =
        "$line:$col\t${ruleId.value}\t${if (canBeAutoCorrected) "auto" else "manual"}\t${detail.replace("\\", "\\\\").replace("\n", "\\n")}"

    private fun Throwable.describe(): String {
        val root = generateSequence(this) { it.cause }.last()
        return "${this::class.simpleName}: ${message.orEmpty().lines().first()} <- ${root::class.simpleName}".replace("\t", " ")
    }

    private fun ruleDir(ruleId: String) = if (ruleId.startsWith("standard:")) ruleId.removePrefix("standard:") else ruleId.replace(':', '_')

    private fun uniqueName(
        dir: File,
        test: String,
    ): String {
        var name = test
        var n = 1
        while (!taken.add("${dir.name}/$name")) name = "$test-${++n}"
        return name
    }
}
