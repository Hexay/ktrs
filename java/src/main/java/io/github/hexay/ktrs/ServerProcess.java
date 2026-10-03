package io.github.hexay.ktrs;

import static java.nio.charset.StandardCharsets.UTF_8;

import java.io.BufferedInputStream;
import java.io.BufferedOutputStream;
import java.io.Closeable;
import java.io.DataInputStream;
import java.io.DataOutputStream;
import java.io.EOFException;
import java.io.IOException;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.TimeUnit;

/** One {@code ktrs serve} child process; not thread-safe (protocol: crates/ktrs-cli/src/serve.rs). */
final class ServerProcess implements Closeable {
    /** The protocol of ktfmt requests; ktlint requests need {@link #KTLINT_PROTOCOL_VERSION}. */
    static final int PROTOCOL_VERSION = 1;
    static final int KTLINT_PROTOCOL_VERSION = 2;

    static final class Response {
        final boolean ok;
        final boolean changed;
        /** The header lines after {@code status} and {@code changed}. */
        final List<String> header;
        final String body;

        Response(boolean ok, boolean changed, List<String> header, String body) {
            this.ok = ok;
            this.changed = changed;
            this.header = header;
            this.body = body;
        }
    }

    private final Process process;
    private final DataInputStream in;
    private final DataOutputStream out;
    private int protocol;

    private ServerProcess(Process process) {
        this.process = process;
        this.in = new DataInputStream(new BufferedInputStream(process.getInputStream()));
        this.out = new DataOutputStream(new BufferedOutputStream(process.getOutputStream()));
    }

    static ServerProcess start(Path executable) throws IOException {
        Process process = new ProcessBuilder(executable.toString(), "serve")
                .redirectError(ProcessBuilder.Redirect.INHERIT)
                .start();
        ServerProcess server = new ServerProcess(process);
        String hello;
        try {
            hello = new String(server.readFrame(), UTF_8);
        } catch (EOFException e) {
            server.close();
            throw new IOException(executable + " exited without starting `ktrs serve` (it needs ktrs 0.2 or later)", e);
        } catch (IOException e) {
            server.close();
            throw e;
        }
        server.protocol = protocol(hello);
        if (server.protocol < PROTOCOL_VERSION) {
            server.close();
            throw new IOException(executable + " speaks an unsupported protocol: " + hello);
        }
        return server;
    }

    /** {@code ktrs-serve <protocol> <version>}: the protocol, or -1. Newer servers answer older requests alike. */
    private static int protocol(String hello) {
        String[] parts = hello.split(" ");
        try {
            return parts.length == 3 && parts[0].equals("ktrs-serve") ? Integer.parseInt(parts[1]) : -1;
        } catch (NumberFormatException e) {
            return -1;
        }
    }

    int protocol() {
        return protocol;
    }

    Response request(String header, String code) throws IOException {
        byte[] payload = (header + "\n" + code).getBytes(UTF_8);
        out.writeInt(payload.length);
        out.write(payload);
        out.flush();
        String response = new String(readFrame(), UTF_8);
        int end = response.indexOf("\n\n");
        if (end < 0) {
            throw new IOException("malformed response from ktrs serve: " + response);
        }
        boolean ok = false;
        boolean changed = false;
        List<String> rest = new ArrayList<>();
        for (String line : response.substring(0, end).split("\n")) {
            if (line.startsWith("status=")) {
                ok = line.equals("status=ok");
            } else if (line.startsWith("changed=")) {
                changed = line.equals("changed=true");
            } else {
                rest.add(line);
            }
        }
        return new Response(ok, changed, rest, response.substring(end + 2));
    }

    private byte[] readFrame() throws IOException {
        int length = in.readInt();
        byte[] frame = new byte[length];
        in.readFully(frame);
        return frame;
    }

    @Override
    public void close() {
        try {
            out.close();
            if (!process.waitFor(2, TimeUnit.SECONDS)) {
                process.destroyForcibly();
            }
        } catch (IOException e) {
            process.destroyForcibly();
        } catch (InterruptedException e) {
            process.destroyForcibly();
            Thread.currentThread().interrupt();
        }
    }
}
