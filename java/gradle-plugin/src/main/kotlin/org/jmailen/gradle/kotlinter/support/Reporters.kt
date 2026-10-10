@file:Suppress("ktlint:standard:enum-entry-name-case")

package org.jmailen.gradle.kotlinter.support

/**
 * kotlinter's reporters. Its ktlint-typed `reporterFor` / `reporterPathFor` are `ktrs ktlint`'s
 * (`crates/ktrs-cli/src/ktlint/kotlinter.rs`).
 */
public enum class ReporterType(public val fileExtension: String) {
    checkstyle("xml"),
    html("html"),
    json("json"),
    plain("txt"),
    sarif("sarif.json"),
}

public fun reporterFileExtension(reporterName: String): String = ReporterType.valueOf(reporterName).fileExtension
