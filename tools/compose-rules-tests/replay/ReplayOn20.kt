@file:Suppress("DEPRECATION")

import com.google.common.jimfs.Configuration
import com.google.common.jimfs.Jimfs
import com.pinterest.ktlint.cli.ruleset.core.api.RuleSetProviderV3
import io.github.ktlint.core.rule.engine.api.Code
import io.github.ktlint.core.rule.engine.api.EditorConfigOverride
import io.github.ktlint.core.rule.engine.api.EditorConfigOverride.Companion.EMPTY_EDITOR_CONFIG_OVERRIDE
import io.github.ktlint.core.rule.engine.api.KtLintRuleEngine
import io.github.ktlint.core.rule.engine.api.LintError
import io.github.ktlint.core.rule.engine.core.api.AutocorrectDecision
import io.github.ktlint.core.rule.engine.core.api.RuleV2Provider
import io.github.ktlint.core.rule.engine.core.api.editorconfig.CODE_STYLE_PROPERTY
import io.github.ktlint.core.rule.engine.core.api.editorconfig.EXPERIMENTAL_RULES_EXECUTION_PROPERTY
import io.github.ktlint.core.rule.engine.core.api.editorconfig.EditorConfigProperty
import io.github.ktlint.core.rule.engine.core.api.editorconfig.createRuleExecutionEditorConfigProperty
import io.github.ktlint.core.rule.engine.core.api.editorconfig.createRuleSetExecutionEditorConfigProperty
import io.github.ktlint.core.ruleset.standard.StandardRuleSetProvider
import java.io.File
import java.nio.file.Files
import java.util.ServiceLoader

/**
 * Replays every case recorded on ktlint 1.8 (`<dir>/<rule-dir>/<case>.options`) on the ktlint 2.0 engine, with the compose
 * rules loaded the way 2.0's CLI loads a 1.x `-R` jar (`RuleSetProviderV3` service -> `RuleProvider.toRuleV2Provider()`), and
 * writes the `<case>.2_0.*` files where 2.0's expectation differs. Convention: tools/compose-rules-tests/extract-goldens.sh.
 * Usage: `ReplayOn20Kt <cases-dir>`; prints one line per differing case and a summary.
 */
fun main(args: Array<String>) {
    val casesDir = File(args.single())
    val providers =
        ServiceLoader.load(RuleSetProviderV3::class.java).flatMap { it.getRuleProviders() }.map { it.toRuleV2Provider() } +
            StandardRuleSetProvider().getRuleProviders()
    val providerById = providers.associateBy { it.ruleId.value }
    var total = 0
    var differingCases = 0
    val differingKinds = sortedMapOf<String, Int>()
    casesDir.walk().filter { it.name.endsWith(".options") }.sortedBy { it.path }.forEach { optionsFile ->
        total++
        val kinds = replay(optionsFile, providerById)
        if (kinds.isNotEmpty()) {
            differingCases++
            println("differs\t${optionsFile.parentFile.name}/${optionsFile.name.removeSuffix(".options")}\t${kinds.joinToString(",")}")
            kinds.forEach { differingKinds.merge(it, 1, Int::plus) }
        }
    }
    println("replayed $total cases on ktlint 2.0: $differingCases differ from 1.8 (by kind: $differingKinds)")
}

private class CaseOptions(
    val rules: List<String>,
    val path: String?,
    val editorConfig: List<Pair<String, String>>,
)

private fun parseOptions(file: File): CaseOptions {
    var rules = emptyList<String>()
    var path: String? = null
    val editorConfig = mutableListOf<Pair<String, String>>()
    file.readLines().filter { it.isNotEmpty() }.forEach { line ->
        val (key, value) = line.split('=', limit = 2)
        when {
            key == "rules" -> rules = value.split(',')
            key == "path" -> path = value
            key.startsWith("ec.") -> editorConfig += key.removePrefix("ec.") to value
        }
    }
    return CaseOptions(rules, path, editorConfig)
}

/** Runs one case on 2.0, writes its `.2_0.*` files, and returns the kinds (lint, format, expected, error) that differ from 1.8. */
private fun replay(
    optionsFile: File,
    providerById: Map<String, RuleV2Provider>,
): List<String> {
    val dir = optionsFile.parentFile
    val name = optionsFile.name.removeSuffix(".options")
    val options = parseOptions(optionsFile)
    val input = listOf("kt", "kts").map { File(dir, "$name.input.$it") }.single { it.isFile }
    val ext = input.name.substringAfterLast('.')
    val content = input.readText()
    val ruleProviders = options.rules.map { providerById[it] ?: error("$name: rule ${it} is not loaded on ktlint 2.0") }.toSet()

    val fileSystem = Jimfs.newFileSystem(Configuration.forCurrentPlatform())
    val code =
        options.path?.let { path ->
            val file = fileSystem.getPath(path).also { Files.createDirectories(it.parent) }
            Files.writeString(file, content)
            Code.fromPath(file)
        } ?: Code.fromSnippet(content, ext == "kts")
    val engine =
        KtLintRuleEngine(
            ruleProviders = ruleProviders,
            editorConfigOverride = editorConfigOverride(options.editorConfig, ruleProviders, providerById.values),
            fileSystem = fileSystem,
        )

    val errors = StringBuilder()
    val lint = StringBuilder()
    runCatching { engine.lint(code) { lint.append(it.row()).append('\n') } }
        .onFailure { errors.append("lint\t").append(it.describe()).append('\n') }
    val format = StringBuilder()
    val formatted =
        runCatching {
            engine.format(code) { e ->
                format.append(e.row()).append('\n')
                AutocorrectDecision.ALLOW_AUTOCORRECT
            }
        }.onFailure { errors.append("format\t").append(it.describe()).append('\n') }
            .getOrNull()
            ?.takeIf { it != content }

    val differs = mutableListOf<String>()
    fun compare(
        kind: String,
        actual: String,
        file18: File,
        file20: File,
    ) {
        file20.delete()
        val expected18 = if (file18.isFile) file18.readText() else ""
        if (actual != expected18) {
            file20.writeText(actual)
            differs += kind
        }
    }
    compare("lint", lint.toString(), File(dir, "$name.lint"), File(dir, "$name.2_0.lint"))
    compare("format", format.toString(), File(dir, "$name.format"), File(dir, "$name.2_0.format"))
    compare("error", errors.toString(), File(dir, "$name.error"), File(dir, "$name.2_0.error"))
    compare("expected", formatted ?: content, File(dir, "$name.expected.$ext").takeIf { it.isFile } ?: input, File(dir, "$name.2_0.expected.$ext"))
    return differs
}

/**
 * Rebuilds the recorded overrides: properties are looked up by name among those any provider declares (tests also override
 * properties the rule under test does not use) and the engine's own.
 */
private fun editorConfigOverride(
    editorConfig: List<Pair<String, String>>,
    ruleProviders: Set<RuleV2Provider>,
    allProviders: Collection<RuleV2Provider>,
): EditorConfigOverride {
    if (editorConfig.isEmpty()) return EMPTY_EDITOR_CONFIG_OVERRIDE
    val known: Map<String, EditorConfigProperty<*>> =
        (
            allProviders.flatMap { it.usesEditorConfigProperties } +
                listOf(EXPERIMENTAL_RULES_EXECUTION_PROPERTY, CODE_STYLE_PROPERTY) +
                ruleProviders.map { it.ruleId.ruleSetId.createRuleSetExecutionEditorConfigProperty() } +
                ruleProviders.map { it.ruleId.createRuleExecutionEditorConfigProperty() }
        ).associateBy { it.name }
    val pairs = editorConfig.map { (name, value) -> (known[name] ?: error("unknown .editorconfig property $name")) to value }
    return EditorConfigOverride.from(*pairs.toTypedArray())
}

private fun LintError.row() =
    "$line:$col\t${ruleId.value}\t${if (canBeAutoCorrected) "auto" else "manual"}\t${detail.replace("\\", "\\\\").replace("\n", "\\n")}"

private fun Throwable.describe(): String {
    val root = generateSequence(this) { it.cause }.last()
    return "${this::class.simpleName}: ${message.orEmpty().lines().first()} <- ${root::class.simpleName}".replace("\t", " ")
}
