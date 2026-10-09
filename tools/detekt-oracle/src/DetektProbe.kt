import dev.detekt.api.Issue
import dev.detekt.tooling.api.AnalysisMode
import dev.detekt.tooling.api.DetektProvider
import dev.detekt.tooling.api.spec.ProcessingSpec
import dev.detekt.tooling.api.spec.RulesSpec
import java.io.File
import java.nio.file.Files
import java.nio.file.Path
import java.util.stream.Collectors
import kotlin.io.path.Path
import kotlin.io.path.absolute
import kotlin.io.path.extension
import kotlin.io.path.invariantSeparatorsPathString

/**
 * The corpus oracle of crates/ktrs-detekt: runs the real detekt engine (light mode) on every `.kt` and `.kts` file under
 * <in-dir> and writes <out-dir>/rows.tsv, one issue per line in analyzer order:
 *   file  line  col  endLine  endCol  start  end  rule  severity  signature  message
 * (`file` relative to <in-dir> with `/`; offsets in UTF-16 units; `\`, tab and newline escaped as `\\`, `\t`,
 * `\n`), plus failed.tsv (`run  <exception chain>`) when the run aborts and run.txt (issue count, seconds).
 * The Rust side writes the same layout: `cargo run -p ktrs-detekt --example detekt_probe`.
 *
 *   DetektProbe <in-dir> <out-dir> [--all-rules] [--sequential] [--build-upon-default-config] [--config <yml>]...
 */
fun main(args: Array<String>) {
    val input = Path(args[0]).absolute().normalize()
    val out = File(args[1]).apply { mkdirs() }
    val flags = args.drop(2)
    val configs = flags.zipWithNext().filter { it.first == "--config" }.map { Path(it.second).absolute() }
    val parallel = "--sequential" !in flags
    val files: Set<Path> = Files.walk(input).use { paths ->
        paths.filter { Files.isRegularFile(it) && (it.extension == "kt" || it.extension == "kts") }
            .map { it.absolute().normalize() }
            .collect(Collectors.toSet())
    }
    val spec = ProcessingSpec {
        logging {
            outputChannel = Discard
            errorChannel = Discard
        }
        project {
            basePath = input
            inputPaths = files
            analysisMode = AnalysisMode.light
        }
        rules {
            activateAllRules = "--all-rules" in flags
            failurePolicy = RulesSpec.FailurePolicy.NeverFail
        }
        config {
            useDefaultConfig = "--build-upon-default-config" in flags
            configPaths = configs
        }
        execution {
            parallelParsing = parallel
            parallelAnalysis = parallel
        }
    }
    val started = System.nanoTime()
    val result = DetektProvider.load().get(spec).run()
    val seconds = (System.nanoTime() - started) / 1e9
    val issues = result.container?.issues.orEmpty()
    File(out, "rows.tsv").bufferedWriter().use { w -> issues.forEach { w.append(it.row()).append('\n') } }
    result.error?.let { error ->
        val chain = generateSequence<Throwable>(error) { it.cause }.joinToString(" <- ") { "${it::class.simpleName}: ${it.message.orEmpty()}" }
        File(out, "failed.tsv").writeText("run\t${chain.escape()}\n")
    }
    File(out, "run.txt").writeText("files=${files.size}\nissues=${issues.size}\nseconds=$seconds\nparallel=$parallel\n")
    println("detekt probe: ${files.size} files, ${issues.size} issues, ${"%.1f".format(seconds)} s")
}

private object Discard : Appendable {
    override fun append(csq: CharSequence?): Appendable = this

    override fun append(csq: CharSequence?, start: Int, end: Int): Appendable = this

    override fun append(c: Char): Appendable = this
}

private fun Issue.row(): String =
    listOf(
        location.path.invariantSeparatorsPathString,
        location.source.line,
        location.source.column,
        location.endSource.line,
        location.endSource.column,
        location.text.start,
        location.text.end,
        ruleInstance.id,
        severity.name,
        entity.signature.escape(),
        message.escape(),
    ).joinToString("\t")

private fun String.escape() = replace("\\", "\\\\").replace("\t", "\\t").replace("\n", "\\n").replace("\r", "\\r")
