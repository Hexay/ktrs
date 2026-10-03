package org.jlleitschuh.gradle.ktlint.reporter

import java.io.Serializable

/** ktlint-gradle's built-in reporters: the ktlint CLI reporter id, report file extension and options. */
public enum class ReporterType(
    public val reporterName: String,
    public val fileExtension: String,
    public val options: List<String>,
) : Serializable {
    PLAIN("plain", "txt", emptyList()),
    PLAIN_GROUP_BY_FILE("plain", "txt", listOf("group_by_file")),
    CHECKSTYLE("checkstyle", "xml", emptyList()),
    JSON("json", "json", emptyList()),
    SARIF("sarif", "sarif", emptyList()),
    HTML("html", "html", emptyList()),
}
