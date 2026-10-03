package io.github.hexay.ktrs.spotless;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.junit.jupiter.api.Assumptions.assumeTrue;

import com.diffplug.spotless.FormatterStep;
import com.diffplug.spotless.Lint;
import io.github.hexay.ktrs.KtlintOptions;
import io.github.hexay.ktrs.KtrsException;
import java.io.File;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.List;
import java.util.Map;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

class KtrsKtlintStepTest {
    static final Path COMPOSE_JAR = Paths.get("../tools/compose-rules/lib/ktlint-compose-0.6.7-all.jar").toAbsolutePath();

    @TempDir
    Path dir;

    private File file(String name) throws Exception {
        Files.writeString(dir.resolve(".editorconfig"), "root = true\n");
        return dir.resolve(name).toFile();
    }

    /** What the step's format throws, as Spotless records it. */
    static List<Lint> lints(FormatterStep step, String code, File file) {
        RuntimeException e = assertThrows(RuntimeException.class, () -> step.format(code, file));
        assertTrue(e instanceof Lint.Has, () -> "not a lint: " + e);
        return ((Lint.Has) e).getLints();
    }

    @Test
    void formats() throws Exception {
        try (FormatterStep step = KtrsKtlintStep.create()) {
            assertEquals("ktlint", step.getName());
            assertEquals("fun f() {\n    val x = 1\n}\n", step.format("fun  f() {\n  val x = 1\n}\n", file("A.kt")));
        }
    }

    /** Spotless's own KtLintStepTest expectation for kotlin/ktlint/unsolvable.dirty. */
    @Test
    void reportsTheFirstUnfixableViolationLikeSpotless() throws Exception {
        String unsolvable = "import a.*\nimport a.b.c.*\nimport a.b\nimport kotlinx.android.synthetic.main.layout_name.*\n";
        try (FormatterStep step = KtrsKtlintStep.create("1.8.0")) {
            assertEquals(List.of(Lint.atLine(1, "standard:no-empty-file", "File 'unsolvable.dirty' should not be empty")),
                    lints(step, unsolvable, file("unsolvable.dirty")));
        }
    }

    @Test
    void parseErrorsCarryTheirLine() throws Exception {
        try (FormatterStep step = KtrsKtlintStep.create()) {
            File file = file("A.kt");
            RuntimeException e = assertThrows(RuntimeException.class, () -> step.format("val x = 1\nfun f( {\n", file));
            assertTrue(e.getMessage().startsWith("2:"), e.getMessage());
        }
    }

    @Test
    void unsupportedVersionsFailAtCreation() {
        IllegalArgumentException e = assertThrows(IllegalArgumentException.class, () -> KtrsKtlintStep.create("1.5.0"));
        assertTrue(e.getMessage().contains("1.8.0 and 2.0.0-ALPHA-4"), e.getMessage());
    }

    @Test
    void ruleSetsOtherThanComposeRulesFail() throws Exception {
        File jar = dir.resolve("custom.jar").toFile();
        Files.writeString(jar.toPath(), "not compose-rules");
        try (FormatterStep step = KtrsKtlintStep.create(KtlintOptions.defaults().withCustomRuleSets(List.of(jar)))) {
            File file = file("A.kt");
            KtrsException e = assertThrows(KtrsException.class, () -> step.format("val x = 1\n", file));
            assertTrue(e.getMessage().contains("in Spotless, keep `ktlint()` for this rule set"), e.getMessage());
        }
    }

    @Test
    void composeRulesRunNatively() throws Exception {
        assumeTrue(Files.isRegularFile(COMPOSE_JAR), "needs tools/sync-compose-rules.sh");
        KtlintOptions options = KtlintOptions.defaults()
                .withCustomRuleSets(List.of(COMPOSE_JAR.toFile()))
                .withEditorConfigOverride(Map.of("ktlint_function_naming_ignore_when_annotated_with", "Composable"));
        try (FormatterStep step = KtrsKtlintStep.create(options)) {
            List<Lint> lints = lints(step, "@Composable\nfun MyComposable() {\n    Text(\"x\")\n}\n", file("A.kt"));
            assertTrue(lints.get(0).getShortCode().startsWith("compose:"), lints.toString());
        }
    }

    @Test
    void equality() throws Exception {
        KtlintOptions options = KtlintOptions.defaults().withEditorConfigOverride(Map.of("indent_size", 2));
        assertEquals(KtrsKtlintStep.create(options), KtrsKtlintStep.create(options));
        assertNotEquals(KtrsKtlintStep.create(options), KtrsKtlintStep.create(KtlintOptions.defaults()));
        assertNotEquals(KtrsKtlintStep.create(), KtrsKtlintStep.create(KtlintOptions.VERSION_2_0));
        Path defaults = dir.resolve("defaults.editorconfig");
        Files.writeString(defaults, "[*.kt]\nindent_size = 2\n");
        FormatterStep before = KtrsKtlintStep.create(options.withEditorConfigPath(defaults.toFile()));
        assertEquals(before, KtrsKtlintStep.create(options.withEditorConfigPath(defaults.toFile())));
        Files.writeString(defaults, "[*.kt]\nindent_size = 3\nmax_line_length = 80\n");
        assertNotEquals(before, KtrsKtlintStep.create(options.withEditorConfigPath(defaults.toFile())));
    }
}
