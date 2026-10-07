package io.github.hexay.ktrs.maven.ktlint.internal;

/**
 * ktlint's {@code ReporterV2}, for the reporters that run in the Maven JVM ({@code maven}, the site report's
 * model); the others are written by {@code ktrs ktlint} itself.
 */
public interface ReporterV2 {
    default void beforeAll() {}

    default void before(String file) {}

    void onLintError(String file, KtlintCliError ktlintCliError);

    default void after(String file) {}

    default void afterAll() {}
}
