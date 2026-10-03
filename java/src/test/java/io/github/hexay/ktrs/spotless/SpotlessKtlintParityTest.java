package io.github.hexay.ktrs.spotless;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assumptions.assumeTrue;

import com.diffplug.spotless.FileSignature;
import com.diffplug.spotless.FormatterStep;
import com.diffplug.spotless.Lint;
import com.diffplug.spotless.Provisioner;
import com.diffplug.spotless.kotlin.KtLintStep;
import io.github.hexay.ktrs.KtlintOptions;
import io.github.hexay.ktrs.KtlintSamples;
import java.io.File;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.TreeMap;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

/**
 * {@link KtrsKtlintStep} vs Spotless's own {@code ktlint("1.8.0")} step (real ktlint, in this JVM) on the same files
 * and options: the same formatted code, or the same lints, or the same exception message. Needs Gradle's
 * {@code spotless.ktlint.classpath}; inputs: {@link KtlintSamples}.
 */
class SpotlessKtlintParityTest {
    private static final String COMPOSE_RULES = "io.nlopez.compose.rules:ktlint:0.6.7";

    @TempDir
    Path dir;

    @Test
    void defaults() throws Exception {
        compare(Map.of(), null, false);
    }

    @Test
    void overridesSwitchToIntellijIdea() throws Exception {
        compare(Map.of("indent_size", 2, "ktlint_standard_no-wildcard-imports", "disabled", "not_a_property", 1), null, false);
    }

    @Test
    void overriddenCodeStyle() throws Exception {
        compare(Map.of("ktlint_code_style", "ktlint_official", "max_line_length", 80), null, false);
    }

    @Test
    void codeStyleFromTheEditorConfigPath() throws Exception {
        Path defaults = dir.resolve("defaults.editorconfig");
        Files.writeString(defaults, "[*.{kt,kts}]\nktlint_code_style = android_studio\nmax_line_length = 90\n");
        compare(Map.of("indent_size", 4), defaults.toFile(), false);
    }

    @Test
    void composeRules() throws Exception {
        assumeTrue(Files.isRegularFile(KtrsKtlintStepTest.COMPOSE_JAR), "needs tools/sync-compose-rules.sh");
        compare(Map.of(), null, true);
    }

    private void compare(Map<String, Object> override, File editorConfig, boolean compose) throws Exception {
        String classpath = System.getProperty(compose ? "spotless.ktlintCompose.classpath" : "spotless.ktlint.classpath");
        assumeTrue(classpath != null && !classpath.isEmpty(), "needs the Gradle test task's ktlint classpath");
        Provisioner provisioner = (withTransitives, coordinates) -> {
            Set<File> files = new LinkedHashSet<>();
            Arrays.stream(classpath.split(File.pathSeparator)).map(File::new).forEach(files::add);
            return files;
        };
        FileSignature signature = editorConfig == null ? null : FileSignature.signAsList(editorConfig);
        List<String> customRuleSets = compose ? List.of(COMPOSE_RULES) : List.of();
        KtlintOptions options = KtlintOptions.defaults().withEditorConfigOverride(override).withEditorConfigPath(editorConfig)
                .withCustomRuleSets(compose ? List.of(KtrsKtlintStepTest.COMPOSE_JAR.toFile()) : List.of());
        Map<Path, String> inputs = KtlintSamples.write(dir);
        List<String> differences = new ArrayList<>();
        try (FormatterStep spotless = KtLintStep.create("1.8.0", provisioner, signature, new TreeMap<>(override), customRuleSets);
                FormatterStep ktrs = KtrsKtlintStep.create(options)) {
            for (Map.Entry<Path, String> input : inputs.entrySet()) {
                File file = input.getKey().toFile();
                String expected = outcome(spotless, input.getValue(), file);
                String actual = outcome(ktrs, input.getValue(), file);
                if (!expected.equals(actual)) {
                    differences.add(dir.relativize(input.getKey()) + "\n--- spotless\n" + expected + "\n--- ktrs\n" + actual);
                }
            }
        }
        assertEquals(List.of(), differences, differences.size() + " of " + inputs.size() + " files differ");
    }

    private static String outcome(FormatterStep step, String code, File file) {
        try {
            return "formatted:\n" + step.format(code, file);
        } catch (Exception e) {
            return e instanceof Lint.Has ? "lints: " + ((Lint.Has) e).getLints() : "error: " + e.getMessage();
        }
    }
}
