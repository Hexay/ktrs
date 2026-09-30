import io.github.ktlint.core.rule.engine.api.Code
import io.github.ktlint.core.rule.engine.api.KtLintParseException
import io.github.ktlint.core.rule.engine.api.KtLintRuleEngine
import io.github.ktlint.core.rule.engine.api.KtLintRuleException
import io.github.ktlint.core.rule.engine.api.LintError
import io.github.ktlint.core.rule.engine.core.api.AutocorrectDecision
import io.github.ktlint.core.rule.engine.core.api.RuleV2Provider
import io.github.ktlint.core.ruleset.standard.StandardRuleSetProvider
import java.io.File
import java.util.concurrent.Callable
import java.util.concurrent.Executors
import kotlin.system.exitProcess

/**
 * Mutated-tree census and oracle on the real KtLintRuleEngine (see tools/ktlint-oracle/ktlint-probe.sh for usage and
 * research/14-ktlint-probe.md for the output format).
 */
class Options(
    val src: File,
    val out: File,
    val rules: List<String>?,
    val dumps: Boolean,
    val lint: Boolean,
    val isolate: Boolean,
    val threads: Int,
)

class FileResult(val rel: String) {
    var passes: List<PassResult> = emptyList()
    var suppressed = false
    val format = mutableListOf<String>()
    val lint = mutableListOf<String>()
    var failure: String? = null
    var formatNanos = 0L
    var probeNanos = 0L
    var lintNanos = 0L
}

val STANDARD: Set<RuleV2Provider> = StandardRuleSetProvider().getRuleProviders()

fun esc(s: String) = s.replace("\\", "\\\\").replace("\t", "\\t").replace("\n", "\\n")

fun LintError.row() = "$line\t$col\t${ruleId.value}\t${if (canBeAutoCorrected) "auto" else "manual"}\t${esc(detail)}"

fun normalize(text: String) = text.replace("\r\n", "\n").replace("\r", "\n").removePrefix("\uFEFF")

fun engineFor(rules: List<String>?): KtLintRuleEngine {
    val chosen =
        if (rules == null) {
            STANDARD
        } else {
            val byId = STANDARD.associateBy { it.ruleId.value }
            rules.map { byId[it] ?: error("unknown rule $it; known: ${byId.keys.sorted()}") }.toSet()
        }
    return KtLintRuleEngine(ruleProviders = chosen + ProbeRule.PROVIDER)
}

fun write(file: File, text: String) {
    file.parentFile.mkdirs()
    file.writeText(text)
}

fun process(engine: KtLintRuleEngine, file: File, rel: String, out: File, dumps: Boolean, lint: Boolean): FileResult {
    val result = FileResult(rel)
    val code = Code.fromFile(file)
    val run = FileRun(code.filePath.toString(), normalize(code.content), dumps)
    try {
        FileRun.CURRENT.set(run)
        val t0 = System.nanoTime()
        val formatted =
            engine.format(code) { e ->
                result.format += "${run.pass}\t${e.row()}"
                if (e.canBeAutoCorrected) run.autocorrectsInPass++
                AutocorrectDecision.ALLOW_AUTOCORRECT
            }
        result.formatNanos = System.nanoTime() - t0
        FileRun.CURRENT.set(null)
        if (formatted != code.content) write(File(out, "fmt/$rel"), formatted)
        if (lint) {
            val t1 = System.nanoTime()
            engine.lint(Code.fromFile(file)) { e -> result.lint += e.row() }
            result.lintNanos = System.nanoTime() - t1
        }
    } catch (e: KtLintParseException) {
        result.failure = "parse\t${e.line}:${e.col} ${esc(e.message.orEmpty())}"
    } catch (e: KtLintRuleException) {
        result.failure = "rule\t${e.ruleId} ${esc((e.cause ?: e).toString())}"
    } catch (e: Throwable) {
        result.failure = "crash\t${esc(e.toString())}"
    } finally {
        FileRun.CURRENT.set(null)
    }
    result.passes = run.passes
    result.suppressed = run.suppressed
    result.probeNanos = run.probeNanos
    for (p in run.passes) {
        if (p.changed && dumps && p.mutated != null) write(File(out, "mut/$rel.p${p.pass}.txt"), p.mutated)
        if (p.diverged && p.mutated != null && p.reparsed != null) {
            write(File(out, "diff/$rel.p${p.pass}.diff"), hunk(p.mutated, p.reparsed))
            if (dumps) write(File(out, "reparse/$rel.p${p.pass}.txt"), p.reparsed)
        }
    }
    return result
}

fun runAll(engine: KtLintRuleEngine, src: File, files: List<String>, out: File, opts: Options, lint: Boolean): List<FileResult> {
    val pool =
        Executors.newFixedThreadPool(opts.threads) { r -> Thread(null, r, "probe", 512L shl 20).apply { isDaemon = true } }
    try {
        return files
            .map { rel -> pool.submit(Callable { process(engine, File(src, rel), rel, out, opts.dumps, lint) }) }
            .map { it.get() }
    } finally {
        pool.shutdown()
    }
}

fun writeTables(out: File, results: List<FileResult>, wallNanos: Long, header: String) {
    out.mkdirs()
    File(out, "passes.tsv").printWriter().use { w ->
        w.println("file\tpass\tchanged\tdiverged")
        for (r in results) for (p in r.passes) w.println("${r.rel}\t${p.pass}\t${if (p.changed) 1 else 0}\t${if (p.diverged) 1 else 0}")
    }
    File(out, "format.tsv").printWriter().use { w ->
        w.println("file\tpass\tline\tcol\trule\tauto\tdetail")
        for (r in results) for (row in r.format) w.println("${r.rel}\t$row")
    }
    File(out, "lint.tsv").printWriter().use { w ->
        w.println("file\tline\tcol\trule\tauto\tdetail")
        for (r in results) for (row in r.lint) w.println("${r.rel}\t$row")
    }
    File(out, "failed.tsv").printWriter().use { w ->
        for (r in results) {
            r.failure?.let { w.println("${r.rel}\t$it") }
            if (r.suppressed) w.println("${r.rel}\tsuppressed\tprobe suppressed by @Suppress/ktlint directive")
        }
    }
    val ok = results.filter { it.failure == null }
    val changed = ok.filter { r -> r.passes.any { it.changed } }
    val diverged = ok.filter { r -> r.passes.any { it.diverged } }
    val divergedAtEnd = ok.filter { r -> r.passes.lastOrNull()?.diverged == true }
    val byPass =
        (1..4).joinToString(" ") { n ->
            val ps = ok.mapNotNull { r -> r.passes.firstOrNull { it.pass == n } }
            "p$n=${ps.count { it.changed }}/${ps.count { it.diverged }}"
        }
    val s = 1e9
    val summary =
        """
        $header
        files ${results.size}, failed ${results.count { it.failure != null }}, probe-suppressed ${results.count { it.suppressed }}
        files changed ${changed.size}, diverged (any pass) ${diverged.size}, diverged after last pass ${divergedAtEnd.size}
        passes changed/diverged by pass number: $byPass
        non-convergent (3 mutating passes) ${ok.count { r -> r.passes.count { it.changed } >= 3 }}
        wall ${"%.1f".format(wallNanos / s)} s; per-file time sums: format ${"%.1f".format(ok.sumOf { it.formatNanos } / s)} s (probe ${
            "%.1f".format(ok.sumOf { it.probeNanos } / s)
        } s of it), lint ${"%.1f".format(ok.sumOf { it.lintNanos } / s)} s
        """.trimIndent()
    File(out, "summary.txt").writeText(summary + "\n")
    println(summary)
}

/** Reruns {rule, probe} on the files where [census] saw that rule autocorrect, to attribute divergences. */
fun isolate(opts: Options, files: Set<String>, census: List<FileResult>) {
    val byRule = sortedMapOf<String, MutableSet<String>>()
    for (r in census) for (row in r.format) {
        val cols = row.split('\t')
        if (cols[4] == "auto" && cols[3].startsWith("standard:") && r.rel in files) byRule.getOrPut(cols[3]) { sortedSetOf() } += r.rel
    }
    val table = File(opts.out, "isolate.tsv").printWriter()
    table.println("rule\tfiles\tchanged\tdiverged\tdiverged_passes\tfailed")
    for ((rule, ruleFiles) in byRule) {
        val out = File(opts.out, "isolate/${rule.substringAfter(':')}")
        val results = runAll(engineFor(listOf(rule)), opts.src, ruleFiles.toList(), out, opts, lint = false)
        val ok = results.filter { it.failure == null }
        table.println(
            "$rule\t${results.size}\t${ok.count { r -> r.passes.any { it.changed } }}\t${ok.count { r -> r.passes.any { it.diverged } }}\t" +
                "${ok.sumOf { r -> r.passes.count { it.diverged && it.reparsed != null } }}\t${results.size - ok.size}",
        )
        table.flush()
        println("isolate $rule: ${results.size} files")
    }
    table.close()
}

fun parseArgs(args: Array<String>): Options {
    if (args.size < 2) {
        System.err.println("usage: KtlintProbe <src-dir> <out-dir> [--rules a,b] [--dumps] [--no-lint] [--isolate] [--threads N]")
        exitProcess(2)
    }
    var rules: List<String>? = null
    var dumps = false
    var lint = true
    var isolate = false
    var threads = Runtime.getRuntime().availableProcessors()
    var i = 2
    while (i < args.size) {
        when (args[i]) {
            "--rules" -> rules = args[++i].split(',')
            "--dumps" -> dumps = true
            "--no-lint" -> lint = false
            "--isolate" -> isolate = true
            "--threads" -> threads = args[++i].toInt()
            else -> error("unknown option ${args[i]}")
        }
        i++
    }
    return Options(File(args[0]).absoluteFile, File(args[1]).absoluteFile, rules, dumps, lint, isolate, threads)
}

fun main(args: Array<String>) {
    val opts = parseArgs(args)
    val files =
        opts.src
            .walkTopDown()
            .filter { it.isFile && (it.name.endsWith(".kt") || it.name.endsWith(".kts")) }
            .map { it.relativeTo(opts.src).invariantSeparatorsPath }
            .sorted()
            .toList()
    val t0 = System.nanoTime()
    val results = runAll(engineFor(opts.rules), opts.src, files, opts.out, opts, opts.lint)
    writeTables(opts.out, results, System.nanoTime() - t0, "ktlint 2.0.0-ALPHA-4 probe, rules=${opts.rules?.joinToString(",") ?: "standard (all)"}")
    if (opts.isolate) {
        val t1 = System.nanoTime()
        isolate(opts, files.toSet(), results)
        println("isolate wall ${"%.1f".format((System.nanoTime() - t1) / 1e9)} s")
    }
    exitProcess(0)
}
