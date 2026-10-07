package io.github.hexay.ktrs.maven.ktlint.internal;

final class FileLintError {
    final String file;
    final int line;
    final int col;
    final String ruleId;
    final String detail;

    FileLintError(String file, KtlintCliError lintError) {
        this.file = file;
        this.line = lintError.line;
        this.col = lintError.col;
        this.ruleId = lintError.ruleId;
        this.detail = lintError.detail;
    }

    int getLine() {
        return line;
    }

    int getCol() {
        return col;
    }

    String getRuleId() {
        return ruleId;
    }
}
