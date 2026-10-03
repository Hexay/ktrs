package org.jlleitschuh.gradle.ktlint.reporter

import java.io.Serializable

/**
 * A 3rd party reporter from the `ktlintReporter` configuration.
 *
 * @param name required for Groovy interop, same as [reporterId]
 * @param reporterId the id the reporter's `ReporterProviderV2` exposes
 * @param fileExtension generated report file extension
 * @param dependency reporter dependency notation, such as `"some.group:reporter:0.1.0"` or
 *   `project(":custom:reporter")`
 */
public data class CustomReporter(
    val name: String,
    val reporterId: String = name,
    var fileExtension: String = reporterId,
    @Transient var dependency: Any? = null,
) : Serializable {
    private companion object {
        private const val serialVersionUID: Long = 2012775L
    }
}
