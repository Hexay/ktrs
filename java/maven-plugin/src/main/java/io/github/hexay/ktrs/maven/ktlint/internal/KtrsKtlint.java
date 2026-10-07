package io.github.hexay.ktrs.maven.ktlint.internal;

import io.github.hexay.ktrs.KtrsExecutable;
import java.io.ByteArrayOutputStream;
import java.io.File;
import java.io.IOException;
import java.io.InputStream;
import java.io.PrintStream;
import java.io.UncheckedIOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.util.ArrayList;
import java.util.List;
import java.util.regex.Pattern;
import java.util.stream.Collectors;
import org.apache.maven.plugin.MojoExecutionException;
import org.apache.maven.plugin.logging.Log;

/**
 * One {@code ktrs ktlint} process (the {@code ktlint} drop-in) over a goal's files, in the module's base directory.
 * A run that loads a rule set or reporter JAR ktrs can't run natively is handed by ktrs to the real ktlint jar, which
 * finds {@code java} through {@code JAVA_HOME}: the Maven JVM's. The binary: {@code -Dktrs.executable}, else the
 * one bundled in {@code io.github.hexay:ktrs}.
 */
final class KtrsKtlint {
    // CreateProcess allows 32767 characters; leave room for the binary and the options.
    private static final int MAX_INLINE_LENGTH = 24_000;
    // ktlint's CLI logs to stdout, beside the reporters that have no output file.
    private static final Pattern LOG_LINE = Pattern.compile("\\d\\d:\\d\\d:\\d\\d\\.\\d{3} \\[[^\\]]*\\] (TRACE|DEBUG|INFO|WARN|ERROR) ");

    private final File workingDir;
    private final File tempDir;
    private final Log log;

    KtrsKtlint(File workingDir, File tempDir, Log log) {
        this.workingDir = workingDir;
        this.tempDir = tempDir;
        this.log = log;
    }

    /**
     * Runs {@code ktrs ktlint <options> <files>}, copying its stdout (reporters without an output file) to
     * {@code System.out}; returns the exit code: 0, or 1 for lint errors.
     */
    int run(List<String> options, List<File> files) throws MojoExecutionException {
        try {
            List<String> command = new ArrayList<>();
            command.add(KtrsExecutable.locate().toString());
            command.add("ktlint");
            command.addAll(options);
            command.addAll(fileArguments(files));
            log.debug("Running " + String.join(" ", command));
            ProcessBuilder builder = new ProcessBuilder(command).directory(workingDir);
            builder.environment().put("JAVA_HOME", System.getProperty("java.home"));
            Process process = builder.start();
            process.getOutputStream().close();
            ByteArrayOutputStream stderr = new ByteArrayOutputStream();
            Thread errReader = new Thread(() -> copy(process.getErrorStream(), stderr));
            errReader.start();
            ByteArrayOutputStream stdout = new ByteArrayOutputStream();
            copy(process.getInputStream(), stdout);
            int exitCode = process.waitFor();
            errReader.join();
            StringBuilder logged = new StringBuilder(stderr.toString(StandardCharsets.UTF_8));
            PrintStream out = System.out;
            for (String line : stdout.toString(StandardCharsets.UTF_8).split("(?<=\n)")) {
                if (LOG_LINE.matcher(line).lookingAt()) logged.append(line);
                else out.print(line);
            }
            out.flush();
            String errors = logged.toString().trim();
            if (!errors.isEmpty()) log.debug(errors);
            if (exitCode != 0 && exitCode != 1) {
                throw new MojoExecutionException("ktrs ktlint failed with exit code " + exitCode + ":\n" + errors);
            }
            return exitCode;
        } catch (IOException e) {
            throw new MojoExecutionException("ktrs ktlint could not run: " + e.getMessage(), e);
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
            throw new MojoExecutionException("ktrs ktlint was interrupted", e);
        }
    }

    /**
     * The files, through an argfile when the command line could get too long for Windows. No files: a path that
     * matches none, since ktlint without file arguments lints the working directory.
     */
    private List<String> fileArguments(List<File> files) throws IOException {
        if (files.isEmpty()) return List.of(new File(tempDir, "no-files.kt").getAbsolutePath());
        List<String> paths = files.stream().map(File::getAbsolutePath).collect(Collectors.toList());
        if (paths.stream().mapToInt(p -> p.length() + 1).sum() < MAX_INLINE_LENGTH) return paths;
        File argfile = new File(tempDir, "files.args");
        String text = paths.stream()
                .map(p -> "\"" + p.replace("\\", "\\\\").replace("\"", "\\\"") + "\"")
                .collect(Collectors.joining("\n"));
        Files.writeString(argfile.toPath(), text, StandardCharsets.UTF_8);
        return List.of("@" + argfile.getAbsolutePath());
    }

    private static void copy(InputStream in, ByteArrayOutputStream out) {
        try (in) {
            in.transferTo(out);
        } catch (IOException e) {
            throw new UncheckedIOException(e);
        }
    }
}
