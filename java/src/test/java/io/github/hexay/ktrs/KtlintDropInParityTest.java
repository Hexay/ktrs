package io.github.hexay.ktrs;

import static java.nio.charset.StandardCharsets.UTF_8;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.regex.Matcher;
import java.util.regex.Pattern;
import org.junit.jupiter.api.io.TempDir;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.ValueSource;

/** {@link Ktrs#ktlint} vs the {@code ktlint} drop-in's {@code -F} on the same files: same code, same unfixed violations. */
class KtlintDropInParityTest {
    private static final Pattern PLAIN = Pattern.compile("(.+?):(\\d+):(\\d+): (.*) \\(([^()]*)\\)");
    /** A rule crash: its row's detail spans several lines. */
    private static final Pattern INTERNAL_ERROR = Pattern.compile("(.+?):\\d+:\\d+: Internal Error .*");
    private static final String NOT_FIXED = " (cannot be auto-corrected)";

    @ParameterizedTest
    @ValueSource(strings = {KtlintOptions.DEFAULT_VERSION, KtlintOptions.VERSION_2_0})
    void sameAsTheDropInWithFormat(String version, @TempDir Path dir) throws Exception {
        Path executable = Paths.get(System.getProperty("ktrs.executable"));
        Map<Path, String> inputs = KtlintSamples.write(dir);
        KtlintOptions options = KtlintOptions.of(version);
        Map<String, Object> ours = new LinkedHashMap<>();
        try (Ktrs ktrs = Ktrs.create(executable)) {
            for (Map.Entry<Path, String> input : inputs.entrySet()) {
                try {
                    ours.put(relative(dir, input.getKey()), ktrs.ktlint(input.getValue(), options, input.getKey()));
                } catch (KtrsException e) {
                    ours.put(relative(dir, input.getKey()), e.getMessage());
                }
            }
        }
        Map<String, Set<String>> reported = dropInFormat(executable, dir, version.equals(KtlintOptions.VERSION_2_0) ? "2.0" : "1.8");
        int compared = 0;
        for (Map.Entry<Path, String> input : inputs.entrySet()) {
            String name = relative(dir, input.getKey());
            Object result = ours.get(name);
            Set<String> theirs = reported.getOrDefault(name, Set.of());
            if (result instanceof String) {
                // The drop-in reports a parse error or rule crash as a row; ktrs serve as an error.
                assertTrue(theirs.stream().anyMatch(v -> v.contains("Not a valid Kotlin file") || v.contains("Internal Error")),
                        name + ": " + result + " vs " + theirs);
                continue;
            }
            KtlintResult ktlint = (KtlintResult) result;
            assertEquals(new String(Files.readAllBytes(input.getKey()), UTF_8), ktlint.code(), name);
            Set<String> expected = new LinkedHashSet<>();
            ktlint.violations().forEach(v -> expected.add(v.line() + ":" + v.col() + " " + v.detail() + " (" + v.ruleId() + ")"));
            assertEquals(expected, theirs, name);
            compared++;
        }
        assertTrue(compared > 0);
    }

    /** Runs {@code ktlint -F} in {@code dir}: the distinct violations it reports, per file. */
    private static Map<String, Set<String>> dropInFormat(Path ktrs, Path dir, String version) throws Exception {
        String name = ktrs.getFileName().toString().replace("ktrs", "ktlint");
        List<String> command = new ArrayList<>(List.of(ktrs.resolveSibling(name).toString(), "-F", "--relative",
                "--ktlint-version=" + version, "--reporter=plain"));
        Process process = new ProcessBuilder(command).directory(dir.toFile()).redirectError(ProcessBuilder.Redirect.DISCARD).start();
        String out = new String(process.getInputStream().readAllBytes(), UTF_8);
        process.waitFor();
        Map<String, Set<String>> reported = new LinkedHashMap<>();
        for (String line : out.split("\r?\n")) {
            Matcher m = PLAIN.matcher(line);
            Matcher crash = INTERNAL_ERROR.matcher(line);
            if (crash.matches()) {
                reported.computeIfAbsent(crash.group(1).replace('\\', '/'), k -> new LinkedHashSet<>()).add(line);
            } else if (m.matches()) {
                String detail = m.group(4).endsWith(NOT_FIXED) ? m.group(4).substring(0, m.group(4).length() - NOT_FIXED.length()) : m.group(4);
                reported.computeIfAbsent(m.group(1).replace('\\', '/'), k -> new LinkedHashSet<>())
                        .add(m.group(2) + ":" + m.group(3) + " " + detail + " (" + m.group(5) + ")");
            }
        }
        return reported;
    }

    private static String relative(Path dir, Path file) {
        return dir.relativize(file).toString().replace('\\', '/');
    }
}
