package io.github.hexay.ktrs.gradle.ktlint

import java.io.File

/** One row of ktlint's `json` report. An empty [rule] marks a file ktlint could not lint. */
internal data class ReportedError(
    val file: String,
    val line: Int,
    val column: Int,
    val message: String,
    val rule: String,
) {
    /** The file's absolute path; with `--relative` the report has it relative to [workingDir]. */
    fun absolutePath(workingDir: File): String =
        File(file).let { if (it.isAbsolute) it else File(workingDir, file) }.absolutePath
}

/**
 * Reads the report of ktlint's `json` reporter (`ktlint-cli-reporter-json`), whose layout is fixed:
 * one `"key": value` per line.
 */
internal object JsonReport {

    fun read(report: File): List<ReportedError> {
        val errors = mutableListOf<ReportedError>()
        var file = ""
        val fields = mutableMapOf<String, String>()
        for (rawLine in report.readLines()) {
            val line = rawLine.trim()
            val key = line.substringAfter('"', "").substringBefore('"', "")
            val value = line.substringAfter("\": ", "").removeSuffix(",")
            when (key) {
                "file" -> file = unquote(value)
                "line", "column", "message" -> fields[key] = value
                "rule" -> {
                    errors +=
                        ReportedError(
                            file,
                            fields.getValue("line").toInt(),
                            fields.getValue("column").toInt(),
                            unquote(fields.getValue("message")),
                            unquote(value),
                        )
                    fields.clear()
                }
            }
        }
        return errors
    }

    /** A JSON string literal as the reporter escapes it (`\\`, `\"`, `\b`, `\n`, `\r`, `\t`). */
    private fun unquote(literal: String): String {
        val body = literal.removePrefix("\"").removeSuffix("\"")
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
}
