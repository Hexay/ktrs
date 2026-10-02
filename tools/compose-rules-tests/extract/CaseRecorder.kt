package com.pinterest.ktlint.test

import com.pinterest.ktlint.rule.engine.api.Code
import com.pinterest.ktlint.rule.engine.api.EditorConfigOverride
import com.pinterest.ktlint.rule.engine.api.EditorConfigOverride.Companion.EMPTY_EDITOR_CONFIG_OVERRIDE
import com.pinterest.ktlint.rule.engine.api.KtLintRuleEngine
import com.pinterest.ktlint.rule.engine.api.LintError
import com.pinterest.ktlint.rule.engine.core.api.AutocorrectDecision
import com.pinterest.ktlint.rule.engine.core.api.RuleProvider
import io.github.ktlint.core.test.CaseNaming
import java.io.File
import java.nio.file.FileSystem
import java.nio.file.FileSystems

/**
 * ktlint 1.8 port of tools/ktlint-tests/extract/CaseRecorder.kt, called from the patched `KtLintAssertThatAssertable.init` and
 * from [RecordingKtLintRuleEngine] (see tools/compose-rules-tests/extract-goldens.sh): writes one golden case per distinct
 * (rules, path, editorconfig, code) of the running test into `-Dgolden.out`, the expectations computed by the real engine.
 * The first provider names the case directory.
 */
public object CaseRecorder {
    private val out = System.getProperty("golden.out")?.let(::File)
    private val seen = mutableSetOf<String>()
    private val taken = mutableSetOf<String>()

    @JvmStatic
    public fun record(
        providers: Set<RuleProvider>,
        code: Code,
        editorConfigOverride: EditorConfigOverride,
        fileSystem: FileSystem,
    ) {
        val out = out ?: return
        val test = CaseNaming.CURRENT.get() ?: "outside-test"
        val options =
            buildString {
                append("ktlint=1.8\n")
                append("rules=").append(providers.joinToString(",") { it.ruleId.value }).append('\n')
                code.filePath?.let { append("path=").append(it.toString().replace('\\', '/')).append('\n') }
                for ((property, value) in editorConfigOverride.properties) {
                    append("ec.").append(property.name).append('=').append(value.source).append('\n')
                }
            }
        if (!seen.add("$test\u0000$options\u0000${code.script}\u0000${code.content}")) return
        val dir = File(out, providers.first().ruleId.value.replace(':', '_')).apply { mkdirs() }
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

/**
 * Stands in for `KtLintRuleEngine(...)` in tests that drive the engine directly instead of through KtLintAssertThat
 * (extract-goldens.sh rewrites the constructor call): records each linted/formatted [Code], then delegates.
 */
public class RecordingKtLintRuleEngine(
    private val ruleProviders: Set<RuleProvider>,
    private val editorConfigOverride: EditorConfigOverride = EMPTY_EDITOR_CONFIG_OVERRIDE,
) {
    private val engine = KtLintRuleEngine(ruleProviders = ruleProviders, editorConfigOverride = editorConfigOverride)

    public fun lint(
        code: Code,
        callback: (LintError) -> Unit = { },
    ) {
        CaseRecorder.record(ruleProviders, code, editorConfigOverride, FileSystems.getDefault())
        engine.lint(code, callback)
    }

    public fun format(
        code: Code,
        callback: (LintError) -> AutocorrectDecision,
    ): String {
        CaseRecorder.record(ruleProviders, code, editorConfigOverride, FileSystems.getDefault())
        return engine.format(code, callback = callback)
    }
}
