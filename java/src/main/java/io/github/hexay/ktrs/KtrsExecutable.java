package io.github.hexay.ktrs;

import java.nio.file.Path;

/** The {@code ktrs} binary, for callers that run it as a command (such as {@code ktrs ktlint}). */
public final class KtrsExecutable {
    private KtrsExecutable() {}

    /** {@code -Dktrs.executable}, else the binary bundled for this platform, extracted on first use. */
    public static Path locate() {
        return NativeBinary.locate();
    }
}
