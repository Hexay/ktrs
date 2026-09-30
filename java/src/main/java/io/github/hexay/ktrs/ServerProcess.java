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
import java.util.concurrent.TimeUnit;

/** One {@code ktrs serve} child process; not thread-safe (protocol: crates/ktrs-cli/src/serve.rs). */
final class ServerProcess implements Closeable {
    static final int PROTOCOL_VERSION = 1;

    static final class Response {
        final boolean ok;
        final boolean changed;
        final String body;

        Response(boolean ok, boolean changed, String body) {
            this.ok = ok;
            this.changed = changed;
            this.body = body;
        }
    }

    private final Process process;
    private final DataInputStream in;
    private final DataOutputStream out;

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
        if (!hello.startsWith("ktrs-serve " + PROTOCOL_VERSION + " ")) {
            server.close();
            throw new IOException(executable + " speaks an unsupported protocol: " + hello);
        }
        return server;
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
        for (String line : response.substring(0, end).split("\n")) {
            ok |= line.equals("status=ok");
            changed |= line.equals("changed=true");
        }
        return new Response(ok, changed, response.substring(end + 2));
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
