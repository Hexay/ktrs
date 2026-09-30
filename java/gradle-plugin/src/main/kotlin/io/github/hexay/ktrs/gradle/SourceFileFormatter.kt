package io.github.hexay.ktrs.gradle

import com.ncorti.ktfmt.gradle.FormattingOptionsBean
import com.ncorti.ktfmt.gradle.util.KtfmtDiffer
import com.ncorti.ktfmt.gradle.util.d
import com.ncorti.ktfmt.gradle.util.e
import com.ncorti.ktfmt.gradle.util.i
import io.github.hexay.ktrs.KtrsOptions
import java.io.File
import org.gradle.api.logging.Logger

internal sealed class KtfmtFormatResult(open val input: File) {
    data class KtfmtFormatSuccess(override val input: File, val wasCorrectlyFormatted: Boolean) :
        KtfmtFormatResult(input)

    data class KtfmtFormatFailure(override val input: File) : KtfmtFormatResult(input)

    data class KtfmtFormatSkipped(override val input: File) : KtfmtFormatResult(input)
}

/** Port of ktfmt-gradle's `KtfmtWorkAction`: checks or reformats one file, logging as it did. */
internal class SourceFileFormatter(
    private val service: KtrsBuildService,
    options: FormattingOptionsBean,
    private val includedFiles: Set<File>,
    private val reformatFiles: Boolean,
    private val logger: Logger,
) {
    private val ktrsOptions = options.toKtrsOptions()

    fun process(sourceFile: File): KtfmtFormatResult {
        if (shouldSkipFile(sourceFile)) {
            logger.i("Skipping format for $sourceFile because it is not included")
            return KtfmtFormatResult.KtfmtFormatSkipped(sourceFile)
        }

        logger.d("Checking format for $sourceFile")

        return runCatching {
                val originalContent = sourceFile.readText()

                val formattedContent = service.format(originalContent, ktrsOptions, sourceFile)

                if (originalContent == formattedContent) {
                    logger.i("Valid formatting for: $sourceFile")
                    return@runCatching KtfmtFormatResult.KtfmtFormatSuccess(sourceFile, true)
                }

                if (reformatFiles) {
                    logger.i("Reformatting $sourceFile")
                    sourceFile.writeText(formattedContent)
                    return@runCatching KtfmtFormatResult.KtfmtFormatSuccess(sourceFile, false)
                }

                logger.e("Invalid formatting for: $sourceFile")
                KtfmtDiffer.printDiff(KtfmtDiffer.computeDiff(sourceFile, formattedContent), logger)
                KtfmtFormatResult.KtfmtFormatSuccess(sourceFile, false)
            }
            .onFailure {
                logger.e("Failed to format file: $sourceFile (reason = ${it.message})")
                logger.d("Failed to format file: $sourceFile", it)
            }
            .getOrElse { KtfmtFormatResult.KtfmtFormatFailure(sourceFile) }
    }

    private fun shouldSkipFile(sourceFile: File): Boolean {
        if (includedFiles.isEmpty()) return false

        return sourceFile.canonicalFile !in includedFiles
    }
}

/** Meta style plus all five explicit values, so only ktfmt-gradle's options matter; no `.editorconfig`. */
internal fun FormattingOptionsBean.toKtrsOptions(): KtrsOptions =
    KtrsOptions.meta()
        .withMaxWidth(maxWidth)
        .withBlockIndent(blockIndent)
        .withContinuationIndent(continuationIndent)
        .withRemoveUnusedImports(removeUnusedImports)
        .withTrailingCommas(KtrsOptions.TrailingCommas.valueOf(trailingCommaManagementStrategy.name))
