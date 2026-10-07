package io.github.hexay.ktrs.maven.ktlint.internal;

import java.util.ArrayList;
import java.util.List;

final class ModelReporter implements ReporterV2 {
    private final List<FileLintError> errors = new ArrayList<>();
    private int fileCount;

    int getFileCount() {
        return fileCount;
    }

    List<FileLintError> getErrors() {
        return errors;
    }

    @Override
    public void onLintError(String file, KtlintCliError ktlintCliError) {
        errors.add(new FileLintError(file, ktlintCliError));
    }

    @Override
    public void after(String file) {
        fileCount++;
    }
}
