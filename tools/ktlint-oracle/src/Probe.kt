import io.github.ktlint.core.rule.engine.core.api.AutocorrectDecision
import io.github.ktlint.core.rule.engine.core.api.KtlintKotlinCompiler
import io.github.ktlint.core.rule.engine.core.api.RuleId
import io.github.ktlint.core.rule.engine.core.api.RuleV2
import io.github.ktlint.core.rule.engine.core.api.RuleV2Provider
import io.github.ktlint.core.rule.engine.core.api.editorconfig.EditorConfig
import org.jetbrains.kotlin.com.intellij.lang.ASTNode
import org.jetbrains.kotlin.com.intellij.psi.impl.DebugUtil

/** Same call and flags as tools/psi-dump (`DebugUtil.psiToString(file, true, false)`). */
fun dump(node: ASTNode): String = DebugUtil.psiToString(node.psi, true, false)

fun freshDump(psiFileName: String, text: String): String = dump(KtlintKotlinCompiler.createPsiFileFromText(psiFileName, text).node)

class PassResult(val pass: Int, val changed: Boolean, val diverged: Boolean, val mutated: String?, val reparsed: String?)

/**
 * Per-file state the probe fills in. `original` is ktlint's own parse input (normalized text), so the pass-1 baseline is
 * a parse with ktlint's factory of exactly what ktlint parsed.
 */
class FileRun(val psiFileName: String, val original: String, val keepDumps: Boolean) {
    var pass = 0
    var autocorrectsInPass = 0
    var probeNanos = 0L
    var suppressed = false
    val passes = mutableListOf<PassResult>()
    private var prevText = original
    private var prevDump: String? = null
    private var prevDiverged = false

    fun endOfPass(root: ASTNode?) {
        val t0 = System.nanoTime()
        if (root == null) {
            suppressed = true
            passes += PassResult(pass, false, false, null, null)
            return
        }
        val text = root.text
        // A rule mutates only after ifAutocorrectAllowed, so no autocorrect + same text = untouched tree (skips two dumps).
        if (autocorrectsInPass == 0 && text == prevText) {
            passes += PassResult(pass, false, prevDiverged, null, null)
        } else {
            val mutated = dump(root)
            val baseline = prevDump ?: freshDump(psiFileName, original)
            val changed = mutated != baseline
            val reparsed = if (changed) freshDump(psiFileName, text) else null
            val diverged = if (changed) mutated != reparsed else prevDiverged
            passes += PassResult(pass, changed, diverged, mutated.takeIf { changed && keepDumps || diverged }, reparsed.takeIf { diverged })
            prevDump = mutated
            prevDiverged = diverged
            prevText = text
        }
        autocorrectsInPass = 0
        probeNanos += System.nanoTime() - t0
    }

    companion object {
        val CURRENT: ThreadLocal<FileRun?> = ThreadLocal()
    }
}

/** Sorts after every standard rule (non-standard sets run last, then by id), so at each node it sees all their edits. */
class ProbeRule : RuleV2(RuleId("zzz:probe"), About()) {
    private var root: ASTNode? = null

    override fun beforeFirstNode(editorConfig: EditorConfig) {
        FileRun.CURRENT.get()?.let { it.pass++ }
    }

    override fun beforeVisitChildNodes(
        node: ASTNode,
        emit: (offset: Int, errorMessage: String, canBeAutoCorrected: Boolean) -> AutocorrectDecision,
    ) {
        if (node.treeParent == null) root = node
        stopTraversalOfAST()
    }

    override fun afterLastNode() {
        FileRun.CURRENT.get()?.endOfPass(root)
    }

    companion object {
        val PROVIDER: RuleV2Provider = RuleV2Provider { ProbeRule() }
    }
}

/**
 * One hunk from the first to the last differing line (common prefix/suffix trimmed), sides capped at [max] lines.
 * Cheap on 100k-line dumps where an LCS diff is not; enough to classify the first divergence.
 */
fun hunk(a: String, b: String, max: Int = 40): String {
    val x = a.lines()
    val y = b.lines()
    var p = 0
    while (p < x.size && p < y.size && x[p] == y[p]) p++
    var s = 0
    while (s < x.size - p && s < y.size - p && x[x.size - 1 - s] == y[y.size - 1 - s]) s++
    val sb = StringBuilder("@@ -${p + 1},${x.size - p - s} +${p + 1},${y.size - p - s} @@ (- mutated, + reparse)\n")
    for (i in maxOf(0, p - 4) until p) sb.append(' ').append(x[i]).append('\n')
    x.subList(p, x.size - s).take(max).forEach { sb.append('-').append(it).append('\n') }
    y.subList(p, y.size - s).take(max).forEach { sb.append('+').append(it).append('\n') }
    return sb.toString()
}
