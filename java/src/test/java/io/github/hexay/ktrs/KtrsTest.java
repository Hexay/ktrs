package io.github.hexay.ktrs;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;
import java.util.concurrent.Future;
import org.junit.jupiter.api.AfterAll;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

class KtrsTest {
    private static Ktrs ktrs;

    @BeforeAll
    static void start() {
        ktrs = Ktrs.create(Paths.get(System.getProperty("ktrs.executable")));
    }

    @AfterAll
    static void stop() {
        ktrs.close();
    }

    @Test
    void formatsWithMetaStyleByDefault() {
        assertEquals("fun f() = 1\n", ktrs.format("fun  f( ) = 1\n"));
    }

    @Test
    void stylesAndOverrides() {
        String code = "fun f() {\n  val x = 1\n}\n";
        assertEquals("fun f() {\n    val x = 1\n}\n", ktrs.format(code, KtrsOptions.kotlinlang()));
        assertEquals("fun f() {\n   val x = 1\n}\n", ktrs.format(code, KtrsOptions.kotlinlang().withBlockIndent(3)));
    }

    @Test
    void keepsUnusedImportsOnRequest() {
        String code = "import a.B\n\nfun f() = 1\n";
        assertEquals("fun f() = 1\n", ktrs.format(code));
        assertEquals(code, ktrs.format(code, KtrsOptions.meta().withRemoveUnusedImports(false)));
    }

    @Test
    void syntaxErrorsThrowKtfmtsMessage() {
        KtrsException e = assertThrows(KtrsException.class,
                () -> ktrs.format("fun f( {\n", KtrsOptions.meta(), Paths.get("src", "A.kt")));
        assertTrue(e.getMessage().startsWith(Paths.get("src", "A.kt") + ":1:"), e.getMessage());
        assertEquals("val x = 1\n", ktrs.format("val x = 1\n"), "the server survives an error");
    }

    @Test
    void appliesEditorConfigAtThePath(@TempDir Path dir) throws Exception {
        Files.writeString(dir.resolve(".editorconfig"), "root = true\n[*.kt]\nindent_size = 8\n");
        String code = "fun f() {\n  val x = 1\n}\n";
        Path file = dir.resolve("A.kt");
        assertEquals("fun f() {\n        val x = 1\n}\n",
                ktrs.format(code, KtrsOptions.meta().withEditorConfig(true), file));
        assertEquals(code, ktrs.format(code, KtrsOptions.meta(), file));
    }

    @Test
    void servesConcurrentCallers() throws Exception {
        ExecutorService pool = Executors.newFixedThreadPool(8);
        try {
            List<Future<String>> results = new ArrayList<>();
            for (int i = 0; i < 64; i++) {
                String code = "fun  f" + i + "( ) = " + i + "\n";
                results.add(pool.submit(() -> ktrs.format(code)));
            }
            for (int i = 0; i < results.size(); i++) {
                assertEquals("fun f" + i + "() = " + i + "\n", results.get(i).get());
            }
        } finally {
            pool.shutdown();
        }
    }

    @Test
    void closedInstancesRefuseWork() {
        Ktrs other = Ktrs.create(Paths.get(System.getProperty("ktrs.executable")));
        assertEquals("val x = 1\n", other.format("val x = 1\n"));
        other.close();
        assertThrows(IllegalStateException.class, () -> other.format("val x = 1\n"));
    }

    @Test
    void optionsRejectLineBreaksAndBadNumbers() {
        assertThrows(IllegalArgumentException.class, () -> KtrsOptions.meta().withMaxWidth(0));
        assertThrows(IllegalArgumentException.class,
                () -> ktrs.format("val x = 1\n", KtrsOptions.meta(), Paths.get("a\nb.kt")));
    }

    @Test
    void platformNames() {
        assertTrue(NativeBinary.platform().matches("(linux|macos|windows)-(x86_64|aarch64)"), NativeBinary.platform());
    }
}
