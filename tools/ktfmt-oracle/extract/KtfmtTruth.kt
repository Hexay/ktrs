// Drop-in replacement for ktfmt's testutil/KtfmtTruth.kt, compiled with the upstream format tests by
// extract-goldens.sh. Instead of asserting, every (input, options, expectation) is written to the
// staging dir in -Dgolden.out for KtfmtOracle to re-verify.
package com.facebook.ktfmt.testutil

import com.facebook.ktfmt.format.Formatter
import com.facebook.ktfmt.format.FormattingOptions
import java.io.File

var defaultTestFormattingOptions: FormattingOptions = Formatter.META_FORMAT

fun assertFormatted(
    code: String,
    formattingOptions: FormattingOptions = defaultTestFormattingOptions,
    deduceMaxWidth: Boolean = false,
) {
  val first = code.lines().first()
  var deducedCode = code
  var maxWidth = FormattingOptions.DEFAULT_MAX_WIDTH
  val isFirstLineAMaxWidthMarker = first.length >= 8 && first.all { it == '-' || it == '/' }
  if (deduceMaxWidth) {
    if (!isFirstLineAMaxWidthMarker) throw RuntimeException("deduceMaxWidth without a marker line")
    deducedCode = code.substring(code.indexOf('\n') + 1)
    maxWidth = first.length
  } else if (isFirstLineAMaxWidthMarker) {
    throw RuntimeException("marker line without deduceMaxWidth")
  }
  GoldenRecorder.record(deducedCode, formattingOptions.copy(maxWidth = maxWidth), deducedCode)
}

fun assertThatFormatting(code: String): FormattedCodeSubject = FormattedCodeSubject(code)

class FormattedCodeSubject(private val code: String) {
  private var options: FormattingOptions = defaultTestFormattingOptions

  fun withOptions(options: FormattingOptions): FormattedCodeSubject {
    this.options = options
    return this
  }

  fun allowTrailingWhitespace(): FormattedCodeSubject = this

  fun isEqualTo(expectedFormatting: String) {
    GoldenRecorder.record(code, options, expectedFormatting)
  }
}

/** Substituted for `Formatter.format(code)` in the copied test sources; the jar decides the outcome. */
fun recordedFormat(code: String): String = recordedFormat(Formatter.META_FORMAT, code)

fun recordedFormat(options: FormattingOptions, code: String): String {
  GoldenRecorder.record(code, options, null)
  return Formatter.format(options, code)
}

object GoldenRecorder {
  private val out = File(checkNotNull(System.getProperty("golden.out")) { "-Dgolden.out not set" })
  private val taken = HashSet<String>()

  fun record(code: String, options: FormattingOptions, expected: String?) {
    val (suite, test) = currentTest()
    val dir = File(out, suite).apply { mkdirs() }
    val base = sanitize(test)
    var name = base
    var n = 2
    while (!taken.add("$suite/${name.lowercase()}")) name = "$base-${n++}"
    File(dir, "$name.input.kt").writeText(code, Charsets.UTF_8)
    File(dir, "$name.options").writeText(serialize(options), Charsets.UTF_8)
    if (expected != null) File(dir, "$name.upstream.kt").writeText(expected, Charsets.UTF_8)
  }

  // The outermost frame of a *Test class is the @Test method (inner frames may be lambdas/helpers).
  private fun currentTest(): Pair<String, String> {
    val frame =
        Thread.currentThread().stackTrace.last {
          it.className.matches(Regex("""com\.facebook\.ktfmt\.format\.\w+Test"""))
        }
    return frame.className.substringAfterLast('.') to frame.methodName
  }

  private fun sanitize(test: String): String =
      test
          .replace(Regex("[^A-Za-z0-9._-]+"), "_")
          .trim('_', '.')
          .take(120)
          .ifEmpty { "case" }

  private fun serialize(o: FormattingOptions): String =
      """
      |maxWidth=${o.maxWidth}
      |blockIndent=${o.blockIndent}
      |continuationIndent=${o.continuationIndent}
      |trailingCommaManagementStrategy=${o.trailingCommaManagementStrategy.name}
      |removeUnusedImports=${o.removeUnusedImports}
      |preserveLambdaBreaks=${o.preserveLambdaBreaks}
      |debuggingPrintOpsAfterFormatting=${o.debuggingPrintOpsAfterFormatting}
      |"""
          .trimMargin()
}
