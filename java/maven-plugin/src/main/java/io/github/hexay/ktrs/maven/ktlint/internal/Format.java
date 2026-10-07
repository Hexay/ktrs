package io.github.hexay.ktrs.maven.ktlint.internal;

import java.io.File;
import java.io.IOException;
import java.io.UncheckedIOException;
import java.nio.file.Files;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import org.apache.maven.artifact.Artifact;
import org.apache.maven.plugin.MojoExecutionException;
import org.apache.maven.plugin.MojoFailureException;
import org.apache.maven.plugin.logging.Log;

public final class Format extends AbstractLintSupport {
    private static final List<String> EXCEPTION_STATUSES = List.of("KOTLIN_PARSE_EXCEPTION", "KTLINT_RULE_ENGINE_EXCEPTION");

    private final String modulePackaging;
    private final List<Sources> sources;
    private int formattedFileCount;

    public Format(Log log, File basedir, String modulePackaging, List<Sources> sources, boolean android,
            boolean enableExperimentalRules, String ktlintVersion, List<Artifact> pluginArtifacts) {
        super(log, basedir, android, enableExperimentalRules, ktlintVersion, pluginArtifacts);
        this.modulePackaging = modulePackaging;
        this.sources = sources;
    }

    public void invoke() throws MojoExecutionException, MojoFailureException {
        withKtrs((ktrs, tempDir) -> {
            List<String> options = commonOptions(tempDir);
            options.add("--format");
            options.add("--relative");
            options.add("--ktrs-relative-to=" + basedir.getAbsolutePath());
            File events = new File(tempDir, "events.txt");
            options.add("--ktrs-gradle-events=" + events.getAbsolutePath());
            options.add("--reporter=plain,output=" + new File(tempDir, "plain.txt"));

            List<File> files = new ArrayList<>();
            forEachSourceFile(modulePackaging, sources, new String[] {"**/*.kt"}, false, files::add);
            Map<File, byte[]> beforeFileContents = new HashMap<>();
            for (File file : files) beforeFileContents.put(file, Files.readAllBytes(file.toPath()));
            ktrs.run(options, files);
            RunErrors runErrors = RunErrors.read(events, basedir);

            forEachSourceFile(modulePackaging, sources, new String[] {"**/*.kt"}, true,
                    file -> formatFile(file, beforeFileContents.get(file), runErrors.of(file)));
            return null;
        });
        log.info(formattedFileCount + " file(s) formatted.");
    }

    /** gantsign's per-file log, from what the run did to {@code file}. */
    private void formatFile(File file, byte[] beforeFileContent, List<KtlintCliError> errors) {
        String baseRelativePath = toRelativeString(file);
        log.debug("checking format: " + baseRelativePath);
        for (KtlintCliError lintError : errors) {
            if (lintError.status != null && EXCEPTION_STATUSES.contains(lintError.status)) {
                log.error(exceptionMessage(lintError.detail));
                return;
            }
            String errMsg = baseRelativePath + ":" + lintError.line + ":" + lintError.col + ": " + lintError.detail;
            log.debug("Format " + (lintError.corrected ? "fixed" : "could not fix") + " > " + errMsg);
        }
        if (!Arrays.equals(beforeFileContent, readAllBytes(file))) {
            log.debug("Format fixed > " + baseRelativePath);
            formattedFileCount++;
        }
    }

    /**
     * gantsign logs the engine exception's message ({@code KtLintParseException}: {@code "3:12 Expecting ')'"}); the run
     * reports ktlint's CLI detail, {@code "Not a valid Kotlin file (3:12 expecting ')')"}, whose message part is
     * lower-cased: its first letter is restored, usually the only capital in the parser's messages.
     */
    static String exceptionMessage(String detail) {
        String prefix = "Not a valid Kotlin file (";
        if (!detail.startsWith(prefix) || !detail.endsWith(")")) return detail;
        String message = detail.substring(prefix.length(), detail.length() - 1);
        int text = message.indexOf(' ') + 1;
        if (text == 0 || text >= message.length()) return message;
        return message.substring(0, text) + Character.toUpperCase(message.charAt(text)) + message.substring(text + 1);
    }

    private static byte[] readAllBytes(File file) {
        try {
            return Files.readAllBytes(file.toPath());
        } catch (IOException e) {
            throw new UncheckedIOException(e);
        }
    }
}
