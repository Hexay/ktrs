package io.github.hexay.ktrs.maven.ktlint.internal;

import java.util.List;

public class AggregatedReporter implements ReporterV2 {
    private final List<ReporterV2> reporters;

    public AggregatedReporter(List<ReporterV2> reporters) {
        this.reporters = reporters;
    }

    @Override
    public void after(String file) {
        reporters.forEach(it -> it.after(file));
    }

    @Override
    public void afterAll() {
        reporters.forEach(ReporterV2::afterAll);
    }

    @Override
    public void before(String file) {
        reporters.forEach(it -> it.before(file));
    }

    @Override
    public void beforeAll() {
        reporters.forEach(ReporterV2::beforeAll);
    }

    @Override
    public void onLintError(String file, KtlintCliError ktlintCliError) {
        reporters.forEach(it -> it.onLintError(file, ktlintCliError));
    }
}
