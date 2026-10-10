package io.github.hexay.ktrs.gradle.ktlint

import java.io.File

/**
 * One error of a lint task's run, as ktlint-gradle's `LintErrorResult` holds it. [status] is ktlint's
 * `KtlintCliError.Status` name, null when unknown; an empty [rule] marks a file ktlint could not lint.
 */
internal data class ReportedError(
    val file: String,
    val line: Int,
    val column: Int,
    val message: String,
    val rule: String,
    val status: String?,
    val corrected: Boolean,
) {
    /** The file's absolute path; with `--relative` the run reports it relative to [base]. */
    fun absolutePath(base: File): String = File(file).let { if (it.isAbsolute) it else File(base, file) }.absolutePath

    /** ktlint-gradle's console row detail (`ConsoleReportWorkAction.logError`). */
    val consoleDetail: String
        get() = if (status == "LINT_CAN_NOT_BE_AUTOCORRECTED") "$message (cannot be auto-corrected)" else message
}

/**
 * Reads a run's errors: `ktrs ktlint --ktrs-gradle-events` output, or, from a run handed to the ktlint jar, the
 * `json` report it was turned into (no statuses there; format details already carry ktlint's suffix).
 */
internal object RunErrors {

    fun read(report: File): List<ReportedError> {
        val lines = report.readLines()
        return if (isEvents(lines)) readEvents(lines) else readJson(lines)
    }

    /** The files the run linted (from a `json` report: those with errors). */
    fun lintedFiles(report: File): List<String> {
        val lines = report.readLines()
        return if (isEvents(lines)) lines.filter { it.startsWith("file\t") }.map { it.substringAfter('\t') }
        else readJson(lines).map { it.file }.distinct()
    }

    fun isEvents(lines: List<String>) = lines.firstOrNull()?.startsWith(EVENTS_HEADER) == true

    private fun readEvents(lines: List<String>): List<ReportedError> =
        lines.filter { it.startsWith("error\t") }.map(::eventError)

    /** The error of an events file's `error` line. */
    fun eventError(line: String): ReportedError {
        val f = line.split('\t', limit = 8)
        return ReportedError(f[1], f[2].toInt(), f[3].toInt(), unescape(f[7]), f[4], f[5], f[6].toBoolean())
    }

    /** ktlint's `json` reporter layout is fixed: one `"key": value` per line. */
    private fun readJson(lines: List<String>): List<ReportedError> {
        val errors = mutableListOf<ReportedError>()
        var file = ""
        val fields = mutableMapOf<String, String>()
        for (rawLine in lines) {
            val line = rawLine.trim()
            val key = line.substringAfter('"', "").substringBefore('"', "")
            val value = line.substringAfter("\": ", "").removeSuffix(",")
            when (key) {
                "file" -> file = unescape(unquote(value))
                "line", "column", "message" -> fields[key] = value
                "rule" -> {
                    errors +=
                        ReportedError(
                            file,
                            fields.getValue("line").toInt(),
                            fields.getValue("column").toInt(),
                            unescape(unquote(fields.getValue("message"))),
                            unescape(unquote(value)),
                            null,
                            false,
                        )
                    fields.clear()
                }
            }
        }
        return errors
    }

    private fun unquote(literal: String): String = literal.removePrefix("\"").removeSuffix("\"")

    /** `\\`, `\"`, `\b`, `\n`, `\r`, `\t` escapes. */
    private fun unescape(body: String): String {
        val out = StringBuilder(body.length)
        var i = 0
        while (i < body.length) {
            val c = body[i]
            if (c == '\\' && i + 1 < body.length) {
                out.append(
                    when (val next = body[i + 1]) {
                        'b' -> '\b'
                        'n' -> '\n'
                        'r' -> '\r'
                        't' -> '\t'
                        else -> next
                    }
                )
                i += 2
            } else {
                out.append(c)
                i++
            }
        }
        return out.toString()
    }

    private const val EVENTS_HEADER = "# ktrs-gradle-events"
}
