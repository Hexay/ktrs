package io.github.hexay.ktrs;

import java.io.IOException;
import java.nio.file.Path;
import java.util.Deque;
import java.util.concurrent.ConcurrentLinkedDeque;

/**
 * Formats Kotlin exactly like ktfmt 0.65 (or ktlint, {@link #ktlint}), by talking to long-lived native
 * {@code ktrs serve} processes: one per concurrent caller, kept for reuse. Thread-safe.
 *
 * <pre>{@code
 * try (Ktrs ktrs = Ktrs.create()) {
 *     String formatted = ktrs.format(code, KtrsOptions.kotlinlang());
 * }
 * }</pre>
 */
public final class Ktrs implements AutoCloseable {
    private static volatile Ktrs shared;

    private final Path executable;
    private final Deque<ServerProcess> idle = new ConcurrentLinkedDeque<>();
    private volatile boolean closed;

    private Ktrs(Path executable) {
        this.executable = executable;
    }

    /** Uses the binary bundled for this platform (or {@code -Dktrs.executable}). */
    public static Ktrs create() {
        return new Ktrs(NativeBinary.locate());
    }

    /** Uses the given {@code ktrs} executable. */
    public static Ktrs create(Path executable) {
        return new Ktrs(executable);
    }

    /** A process-wide instance, closed at JVM shutdown: for build-tool steps that can't own one. */
    public static Ktrs shared() {
        Ktrs instance = shared;
        if (instance == null) {
            synchronized (Ktrs.class) {
                instance = shared;
                if (instance == null) {
                    instance = create();
                    Runtime.getRuntime().addShutdownHook(new Thread(instance::close, "ktrs-shutdown"));
                    shared = instance;
                }
            }
        }
        return instance;
    }

    public String format(String code) {
        return format(code, KtrsOptions.meta(), null);
    }

    public String format(String code, KtrsOptions options) {
        return format(code, options, null);
    }

    /**
     * @param file the code's path, if any: named in error messages, and where {@code .editorconfig}
     *     files are looked up when the options enable them
     * @throws KtrsException if the code does not parse (ktfmt's message) or the server fails
     */
    public String format(String code, KtrsOptions options, Path file) {
        return request(options.header(file), code, ServerProcess.PROTOCOL_VERSION).body;
    }

    /**
     * Formats like Spotless's {@code ktlint()} step with that ktlint version, and returns what could not be
     * autocorrected too.
     *
     * @param file the code's path, if any: where {@code .editorconfig} files are looked up, and what file name
     *     rules check
     * @throws KtrsException if the code does not parse (ktlint's {@code <line>:<col> <message>}), a rule set JAR
     *     can't run natively, or the server fails
     */
    public KtlintResult ktlint(String code, KtlintOptions options, Path file) {
        ServerProcess.Response response = request(options.header(file), code, ServerProcess.KTLINT_PROTOCOL_VERSION);
        return KtlintResult.parse(response.body, response.changed, response.header);
    }

    private ServerProcess.Response request(String header, String code, int protocol) {
        if (closed) {
            throw new IllegalStateException("Ktrs is closed");
        }
        ServerProcess server = idle.pollFirst();
        boolean reusable = false;
        try {
            if (server == null) {
                server = ServerProcess.start(executable);
            }
            if (server.protocol() < protocol) {
                reusable = true;
                throw new KtrsException(executable + " is too old for this request: it speaks `ktrs serve` protocol "
                        + server.protocol() + ", this needs " + protocol);
            }
            ServerProcess.Response response = server.request(header, code);
            reusable = true;
            if (!response.ok) {
                throw new KtrsException(response.body);
            }
            return response;
        } catch (IOException e) {
            throw new KtrsException("ktrs serve failed: " + e.getMessage(), e);
        } finally {
            if (server != null) {
                if (reusable && !closed) {
                    idle.push(server);
                    if (closed) {
                        close();
                    }
                } else {
                    server.close();
                }
            }
        }
    }

    /** Stops the idle server processes; ones in use stop when their request completes. */
    @Override
    public void close() {
        closed = true;
        for (ServerProcess server; (server = idle.pollFirst()) != null; ) {
            server.close();
        }
    }
}
