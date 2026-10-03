package io.github.hexay.ktrs;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.junit.jupiter.api.Assumptions.assumeTrue;

import java.io.File;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.List;
import java.util.Map;
import org.junit.jupiter.api.AfterAll;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

class KtlintTest {
    /** Fetched by tools/sync-compose-rules.sh; the tests that need it are skipped without it. */
    static final Path COMPOSE_JAR = Paths.get("../tools/compose-rules/lib/ktlint-compose-0.6.7-all.jar").toAbsolutePath();

    private static Ktrs ktrs;

    @TempDir
    Path dir;

    @BeforeAll
    static void start() {
        ktrs = Ktrs.create(Paths.get(System.getProperty("ktrs.executable")));
    }

    @AfterAll
    static void stop() {
        ktrs.close();
    }

    private Path file(String editorConfig) throws Exception {
        Files.writeString(dir.resolve(".editorconfig"), "root = true\n" + editorConfig);
        return dir.resolve("A.kt");
    }

    @Test
    void formatsAndReportsWhatCouldNotBeAutocorrected() throws Exception {
        Path file = file("");
        KtlintResult result = ktrs.ktlint("import a.*\n\nfun  f() = a()\n", KtlintOptions.defaults(), file);
        assertEquals("import a.*\n\nfun f() = a()\n", result.code());
        assertTrue(result.changed());
        assertEquals(List.of(new KtlintResult.Violation(1, 1, "standard:no-wildcard-imports", "Wildcard import")),
                result.violations());
        KtlintResult empty = ktrs.ktlint("", KtlintOptions.defaults(), file);
        assertFalse(empty.changed());
        assertEquals("File 'A.kt' should not be empty", empty.violations().get(0).detail());
    }

    @Test
    void overridesFollowSpotless() throws Exception {
        Path file = file("[*.kt]\nindent_size = 2\n");
        String code = "fun f() {\n    val x = 1\n}\n";
        assertEquals("fun f() {\n  val x = 1\n}\n", ktrs.ktlint(code, KtlintOptions.defaults(), file).code());
        KtlintOptions three = KtlintOptions.defaults().withEditorConfigOverride(Map.of("indent_size", 3, "bogus", true));
        assertEquals("fun f() {\n   val x = 1\n}\n", ktrs.ktlint(code, three, file).code());
        KtlintOptions noWildcards = KtlintOptions.defaults()
                .withEditorConfigOverride(Map.of("ktlint_standard_no-wildcard-imports", "disabled"));
        assertTrue(ktrs.ktlint("import a.*\n\nval x = a()\n", noWildcards, file).violations().isEmpty());
    }

    @Test
    void editorConfigPathSuppliesDefaults() throws Exception {
        Path file = file("");
        Path defaults = dir.resolve("defaults.editorconfig");
        Files.writeString(defaults, "[*.kt]\nindent_size = 3\n");
        KtlintOptions options = KtlintOptions.defaults().withEditorConfigPath(defaults.toFile());
        assertEquals("fun f() {\n   val x = 1\n}\n", ktrs.ktlint("fun f() {\n val x = 1\n}\n", options, file).code());
    }

    @Test
    void versions() {
        assertEquals("2.0.0-ALPHA-4", KtlintOptions.of(KtlintOptions.VERSION_2_0).version());
        IllegalArgumentException e = assertThrows(IllegalArgumentException.class, () -> KtlintOptions.of("1.7.1"));
        assertEquals("ktrs matches ktlint 1.8.0 and 2.0.0-ALPHA-4, not 1.7.1", e.getMessage());
    }

    @Test
    void parseErrorsAreKtlintsMessage() throws Exception {
        KtrsException e = assertThrows(KtrsException.class,
                () -> ktrs.ktlint("fun f( {\n", KtlintOptions.defaults(), file("")));
        assertTrue(e.getMessage().matches("1:\\d+ .*"), e.getMessage());
    }

    @Test
    void ruleSetsOtherThanComposeRulesFail() throws Exception {
        File jar = dir.resolve("custom.jar").toFile();
        Files.writeString(jar.toPath(), "not compose-rules");
        KtlintOptions options = KtlintOptions.defaults().withCustomRuleSets(List.of(jar));
        KtrsException e = assertThrows(KtrsException.class, () -> ktrs.ktlint("val x = 1\n", options, file("")));
        assertTrue(e.getMessage().contains("keep `ktlint()` for this rule set"), e.getMessage());
    }

    @Test
    void composeRulesRunNatively() throws Exception {
        assumeTrue(Files.isRegularFile(COMPOSE_JAR), "needs tools/sync-compose-rules.sh");
        KtlintOptions options = KtlintOptions.defaults().withCustomRuleSets(List.of(COMPOSE_JAR.toFile()));
        KtlintResult result = ktrs.ktlint("@Composable\nfun MyComposable() {\n    Text(\"x\")\n}\n", options, file(""));
        assertTrue(result.violations().stream().anyMatch(v -> v.ruleId().startsWith("compose:")), result.violations().toString());
    }

    @Test
    void optionsHeader() {
        KtlintOptions options = KtlintOptions.of(KtlintOptions.VERSION_2_0)
                .withEditorConfigOverride(Map.of("b", 2, "a", "x"));
        assertEquals("tool=ktlint\nktlint-version=2.0\neditorconfig-override=a=x\neditorconfig-override=b=2\npath=A.kt\n",
                options.header(Paths.get("A.kt")));
        assertEquals(options, KtlintOptions.of(KtlintOptions.VERSION_2_0).withEditorConfigOverride(Map.of("a", "x", "b", 2)));
    }

    @Test
    void escapedDetails() {
        KtlintResult result = KtlintResult.parse("", false, List.of("violation=1\t2\ts:r\ta\\nb\\\\c\\td"));
        assertEquals(new KtlintResult.Violation(1, 2, "s:r", "a\nb\\c\td"), result.violations().get(0));
    }
}
