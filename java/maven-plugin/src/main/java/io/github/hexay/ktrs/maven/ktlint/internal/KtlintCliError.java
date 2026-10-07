package io.github.hexay.ktrs.maven.ktlint.internal;

/**
 * ktlint's {@code KtlintCliError} as a run of {@code ktrs ktlint} reports it. {@code status} is the
 * {@code KtlintCliError.Status} name, null when the run was handed to the ktlint jar (whose {@code json} report has
 * none); an empty {@code ruleId} marks a file ktlint could not lint.
 */
public final class KtlintCliError {
    public final int line;
    public final int col;
    public final String ruleId;
    public final String detail;
    public final String status;
    public final boolean corrected;

    public KtlintCliError(int line, int col, String ruleId, String detail, String status, boolean corrected) {
        this.line = line;
        this.col = col;
        this.ruleId = ruleId;
        this.detail = detail;
        this.status = status;
        this.corrected = corrected;
    }
}
