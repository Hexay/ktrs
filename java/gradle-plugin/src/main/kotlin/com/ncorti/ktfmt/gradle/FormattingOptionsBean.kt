package com.ncorti.ktfmt.gradle

import java.io.Serializable

/** The [KtfmtExtension] values a task formats with (a task input). */
public data class FormattingOptionsBean(
    /** ktfmt breaks lines longer than maxWidth. */
    val maxWidth: Int = defaultMaxWidth,

    /** The size of the indent used when a new block is opened, in spaces. */
    val blockIndent: Int = 2,

    /** The size of the indent used when a line is broken because it's too long, in spaces. */
    val continuationIndent: Int = 4,

    /** See [TrailingCommaManagementStrategy]. */
    val trailingCommaManagementStrategy: TrailingCommaManagementStrategy,

    /** Whether ktfmt should remove imports that are not used. */
    val removeUnusedImports: Boolean = true,

    /** Not supported by ktrs: setting it logs a warning and has no other effect. */
    val debuggingPrintOpsAfterFormatting: Boolean = false,
) : Serializable {

    private companion object {
        const val serialVersionUID: Long = 1L
        const val defaultMaxWidth = 100
    }
}
