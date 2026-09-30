package io.github.hexay.ktrs;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.junit.jupiter.api.Assumptions.assumeTrue;

import java.nio.file.Files;
import java.nio.file.Path;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

/** Runs when the build bundles binaries ({@code -PnativeDir=...}); the release workflow does. */
class NativeBinaryTest {
    @Test
    void extractsTheBundledBinaryOnceAndFormatsWithIt(@TempDir Path cache) {
        String name = NativeBinary.platform().startsWith("windows") ? "ktrs.exe" : "ktrs";
        assumeTrue(NativeBinary.class.getResource("native/" + NativeBinary.platform() + "/" + name) != null,
                "no binary bundled for " + NativeBinary.platform());
        String executable = System.clearProperty("ktrs.executable");
        System.setProperty("ktrs.cache.dir", cache.toString());
        try {
            Path extracted = NativeBinary.locate();
            assertTrue(extracted.startsWith(cache) && Files.isExecutable(extracted), extracted.toString());
            assertEquals(extracted, NativeBinary.locate(), "reuses the cached copy");
            try (Ktrs ktrs = Ktrs.create()) {
                assertEquals("fun f() = 1\n", ktrs.format("fun  f( ) = 1\n"));
            }
        } finally {
            System.clearProperty("ktrs.cache.dir");
            if (executable != null) {
                System.setProperty("ktrs.executable", executable);
            }
        }
    }
}
