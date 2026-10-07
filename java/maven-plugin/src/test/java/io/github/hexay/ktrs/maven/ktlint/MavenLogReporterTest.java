package io.github.hexay.ktrs.maven.ktlint;

import static org.junit.jupiter.api.Assertions.assertEquals;

import io.github.hexay.ktrs.maven.ktlint.internal.KtlintCliError;
import java.io.File;
import java.util.List;
import java.util.Map;
import org.junit.jupiter.api.Test;

class MavenLogReporterTest {
    private static final String S = File.separator;
    private static final String FILE = "src" + S + "Example.kt";

    private static List<String> report(Map<String, String> options, String file) {
        RecordingLog log = new RecordingLog();
        MavenLogReporter reporter = new MavenLogReporterProvider().get(log, options);
        reporter.before(file);
        reporter.onLintError(file, new KtlintCliError(1, 1, "standard:a", "First", "LINT_CAN_BE_AUTOCORRECTED", false));
        reporter.onLintError(file, new KtlintCliError(12, 30, "standard:b", "Second", null, false));
        reporter.after(file);
        return log.at("ERROR");
    }

    @Test
    void rowsPerError() {
        assertEquals(List.of("src" + S + "Example.kt:1:1: First", "src" + S + "Example.kt:12:30: Second"),
                report(Map.of("verbose", "false"), FILE));
    }

    @Test
    void verboseAddsTheRuleId() {
        assertEquals(List.of("src" + S + "Example.kt:1:1: First (standard:a)", "src" + S + "Example.kt:12:30: Second (standard:b)"),
                report(Map.of("verbose", "true"), FILE));
    }

    @Test
    void groupByFileWithPad() {
        assertEquals(List.of("src" + S + "Example.kt", " 1:1   First", " 12:30  Second"),
                report(Map.of("group_by_file", "", "pad", "true"), FILE));
    }

    @Test
    void padsTheColumn() {
        assertEquals(List.of("src" + S + "Example.kt:1:1:  First", "src" + S + "Example.kt:12:30: Second"),
                report(Map.of("pad", "true"), FILE));
    }

    @Test
    void aPathWithoutSeparatorRepeatsTheName() {
        assertEquals("script.kts" + S + "script.kts:1:1: First", report(Map.of(), "script.kts").get(0));
    }
}
