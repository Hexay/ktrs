package org.jmailen.gradle.kotlinter.support

import org.gradle.api.file.ConfigurableFileCollection
import org.gradle.api.logging.Logger

/** Upstream resets its worker daemon's engine caches here; every `ktrs ktlint` run reads `.editorconfig` anew. */
internal fun resetEditorconfigCacheIfNeeded(changedEditorconfigFiles: ConfigurableFileCollection, logger: Logger) {
    if (changedEditorconfigFiles.files.any()) {
        logger.info("Editorconfig changed, resetting KtLint caches")
    }
}
