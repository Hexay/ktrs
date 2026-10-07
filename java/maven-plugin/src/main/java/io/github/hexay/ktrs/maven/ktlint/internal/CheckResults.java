package io.github.hexay.ktrs.maven.ktlint.internal;

import java.util.List;

public final class CheckResults {
    final int fileCount;
    final List<FileLintError> errors;

    CheckResults(int fileCount, List<FileLintError> errors) {
        this.fileCount = fileCount;
        this.errors = errors;
    }
}
