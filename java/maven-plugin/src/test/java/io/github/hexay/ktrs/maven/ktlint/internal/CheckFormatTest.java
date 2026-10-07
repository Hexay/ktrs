package io.github.hexay.ktrs.maven.ktlint.internal;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import io.github.hexay.ktrs.maven.ktlint.RecordingLog;
import io.github.hexay.ktrs.maven.ktlint.ReporterConfig;
import java.io.File;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Set;
import org.apache.maven.plugin.MojoExecutionException;
import org.apache.maven.plugin.MojoFailureException;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

/** The goals end to end on the workspace's {@code ktrs} binary ({@code -Dktrs.executable}, set by the build). */
class CheckFormatTest {
    private static final String S = File.separator;

    @TempDir Path basedir;
    private final RecordingLog log = new RecordingLog();

    @BeforeEach
    void writeSources() throws IOException {
        Path main = Files.createDirectories(basedir.resolve("src/main/kotlin"));
        Files.writeString(main.resolve("Fail.kt"), "class Fail{\n    fun test() {   val x = 1 ; println(x) }\n}\n");
        Files.writeString(main.resolve("Clean.kt"), "val foo = \"bar\"\n");
    }

    private List<Sources> sources() {
        return List.of(
                new Sources(true, List.of(basedir.resolve("src/main/kotlin").toString()), Set.of("**/*.kt"), null),
                new Sources(true, List.of(basedir.resolve("src/test/kotlin").toString()), Set.of("**/*.kt"), null));
    }

    private Check check(Set<ReporterConfig> reporters) {
        return new Check(log, basedir.toFile(), "jar", sources(), false, reporters, false, false, "DARK_GRAY", false, true,
                "1.8.0", List.of());
    }

    @Test
    void checkReportsAndFails() throws IOException {
        File plain = basedir.resolve("target/ktlint.txt").toFile();
        Set<ReporterConfig> reporters = new LinkedHashSet<>(List.of(new ReporterConfig("plain", plain, null)));
        MojoFailureException e = assertThrows(MojoFailureException.class, () -> check(reporters).invoke());
        assertEquals("Kotlin source failed ktlint check.", e.getMessage());
        List<String> rows = log.at("ERROR");
        assertTrue(rows.size() > 2, rows.toString());
        assertTrue(rows.stream().allMatch(r -> r.startsWith("src" + S + "main" + S + "kotlin" + S + "Fail.kt:")), rows.toString());
        assertEquals(List.of("Source root doesn't exist: src" + S + "test" + S + "kotlin"), log.at("WARNING"));
        String report = Files.readString(plain.toPath());
        assertTrue(report.startsWith("src" + S + "main" + S + "kotlin" + S + "Fail.kt:"), report);
    }

    @Test
    void unknownReporter() {
        Set<ReporterConfig> reporters = Set.of(new ReporterConfig("nope"));
        MojoFailureException e = assertThrows(MojoFailureException.class, () -> check(reporters).invoke());
        assertEquals("Error: reporter 'nope' wasn't found (available: baseline,checkstyle,json,maven,plain)", e.getMessage());
    }

    @Test
    void formatThenCheckPasses() throws MojoExecutionException, MojoFailureException {
        new Format(log, basedir.toFile(), "jar", sources(), false, false, "1.8.0", List.of()).invoke();
        assertEquals(List.of("1 file(s) formatted."), log.at("INFO"));
        log.lines.clear();
        check(Set.of()).invoke();
        assertEquals(List.of(), log.at("ERROR"));
    }
}
