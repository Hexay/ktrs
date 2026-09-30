package com.ncorti.ktfmt.gradle

import org.gradle.api.provider.Property

/** The `ktfmt { }` extension, as in ktfmt-gradle 0.27.0. Defaults are ktfmt's Meta style. */
@Suppress("UnnecessaryAbstractClass")
public abstract class KtfmtExtension {
    init {
        maxWidth.convention(DEFAULT_MAX_WIDTH)
        blockIndent.convention(DEFAULT_BLOCK_INDENT)
        continuationIndent.convention(DEFAULT_CONTINUATION_INDENT)
        removeUnusedImports.convention(DEFAULT_REMOVE_UNUSED_IMPORTS)
        debuggingPrintOpsAfterFormatting.convention(DEFAULT_DEBUGGING_PRINT_OPTS)
        trailingCommaManagementStrategy.convention(DEFAULT_TRAILING_COMMAS_STRATEGY)
        srcSetPathExclusionPattern.convention(DEFAULT_SRC_SET_PATH_EXCLUSION_PATTERN)
        useClassloaderIsolation.convention(DEFAULT_USE_CLASSLOADER_ISOLATION)
    }

    /** ktfmt breaks lines longer than maxWidth. Default 100. */
    public abstract val maxWidth: Property<Int>

    /** The size of the indent used when a new block is opened, in spaces. Default 2. */
    public abstract val blockIndent: Property<Int>

    /** The size of the indent used when a line is broken because it's too long, in spaces. Default 4. */
    public abstract val continuationIndent: Property<Int>

    /** See [TrailingCommaManagementStrategy]. Default [TrailingCommaManagementStrategy.ONLY_ADD]. */
    public abstract val trailingCommaManagementStrategy: Property<TrailingCommaManagementStrategy>

    /** Whether ktfmt should remove imports that are not used. Default true. */
    public abstract val removeUnusedImports: Property<Boolean>

    /**
     * Source directories whose absolute path matches this regex get no tasks' attention (default: those
     * under a `build` directory). To exclude files inside a source set, use the task's include/exclude.
     */
    public abstract val srcSetPathExclusionPattern: Property<Regex>

    /** Not supported by ktrs: setting it logs a warning and has no other effect. */
    public abstract val debuggingPrintOpsAfterFormatting: Property<Boolean>

    /** Accepted for compatibility and ignored: ktrs formats in native processes, not Gradle workers. */
    public abstract val useClassloaderIsolation: Property<Boolean>

    @Deprecated(
        "This was updated to trailingCommaManagementStrategy and will be removed in a future release.",
        ReplaceWith("trailingCommaManagementStrategy"),
    )
    public var manageTrailingCommas: Boolean
        set(value) {
            trailingCommaManagementStrategy.set(
                if (value) TrailingCommaManagementStrategy.COMPLETE
                else TrailingCommaManagementStrategy.NONE
            )
        }
        get() = trailingCommaManagementStrategy.get() != TrailingCommaManagementStrategy.NONE

    /** Sets the Google style (equivalent to set blockIndent to 2 and continuationIndent to 2). */
    @Suppress("MagicNumber")
    public fun googleStyle() {
        blockIndent.set(2)
        continuationIndent.set(2)
        trailingCommaManagementStrategy.set(TrailingCommaManagementStrategy.COMPLETE)
    }

    /** Sets the KotlinLang style (https://kotlinlang.org/docs/coding-conventions.html). */
    @Suppress("MagicNumber")
    public fun kotlinLangStyle() {
        blockIndent.set(4)
        continuationIndent.set(4)
        trailingCommaManagementStrategy.set(TrailingCommaManagementStrategy.COMPLETE)
    }

    internal fun toFormattingOptions(): FormattingOptionsBean =
        FormattingOptionsBean(
            maxWidth = maxWidth.get(),
            blockIndent = blockIndent.get(),
            continuationIndent = continuationIndent.get(),
            trailingCommaManagementStrategy = trailingCommaManagementStrategy.get(),
            removeUnusedImports = removeUnusedImports.get(),
            debuggingPrintOpsAfterFormatting = debuggingPrintOpsAfterFormatting.get(),
        )

    internal companion object {
        internal const val DEFAULT_MAX_WIDTH: Int = 100
        internal const val DEFAULT_BLOCK_INDENT: Int = 2
        internal const val DEFAULT_CONTINUATION_INDENT: Int = 4
        internal const val DEFAULT_REMOVE_UNUSED_IMPORTS: Boolean = true
        internal const val DEFAULT_DEBUGGING_PRINT_OPTS: Boolean = false
        internal val DEFAULT_TRAILING_COMMAS_STRATEGY: TrailingCommaManagementStrategy =
            TrailingCommaManagementStrategy.ONLY_ADD
        internal const val DEFAULT_USE_CLASSLOADER_ISOLATION: Boolean = false
        internal val DEFAULT_SRC_SET_PATH_EXCLUSION_PATTERN =
            Regex("^(.*[\\\\/])?build([\\\\/].*)?\$")
    }
}
