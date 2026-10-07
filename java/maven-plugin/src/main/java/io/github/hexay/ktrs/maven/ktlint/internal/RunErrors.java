package io.github.hexay.ktrs.maven.ktlint.internal;

import java.io.File;
import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

/**
 * A run's errors by file: {@code ktrs ktlint --ktrs-gradle-events} output, or, from a run handed to the ktlint jar,
 * the {@code json} report it was turned into (no statuses there). Paths in either are relative to the run's
 * working directory.
 */
public final class RunErrors {
    private static final String EVENTS_HEADER = "# ktrs-gradle-events";

    private final Path base;
    private final Map<Path, List<KtlintCliError>> errorsByFile = new HashMap<>();

    private RunErrors(Path base) {
        this.base = base;
    }

    /** The errors of {@code file} (an empty list when it had none). */
    public List<KtlintCliError> of(File file) {
        return errorsByFile.getOrDefault(key(file.getPath()), List.of());
    }

    public static RunErrors read(File report, File workingDir) throws IOException {
        List<String> lines = Files.readAllLines(report.toPath(), StandardCharsets.UTF_8);
        RunErrors errors = new RunErrors(workingDir.toPath().toAbsolutePath());
        if (!lines.isEmpty() && lines.get(0).startsWith(EVENTS_HEADER)) {
            errors.readEvents(lines);
        } else {
            errors.readJson(lines);
        }
        return errors;
    }

    private Path key(String file) {
        return base.resolve(file).normalize();
    }

    private void add(String file, KtlintCliError error) {
        errorsByFile.computeIfAbsent(key(file), k -> new ArrayList<>()).add(error);
    }

    private void readEvents(List<String> lines) {
        for (String line : lines) {
            if (!line.startsWith("error\t")) continue;
            String[] f = line.split("\t", 8);
            add(f[1], new KtlintCliError(Integer.parseInt(f[2]), Integer.parseInt(f[3]), f[4], unescape(f[7]), f[5],
                    Boolean.parseBoolean(f[6])));
        }
    }

    /** ktlint's {@code json} reporter layout is fixed: one {@code "key": value} per line. */
    private void readJson(List<String> lines) {
        String file = "";
        Map<String, String> fields = new HashMap<>();
        for (String rawLine : lines) {
            String line = rawLine.trim();
            int keyStart = line.indexOf('"');
            int keyEnd = keyStart < 0 ? -1 : line.indexOf('"', keyStart + 1);
            if (keyEnd < 0) continue;
            String key = line.substring(keyStart + 1, keyEnd);
            int valueStart = line.indexOf("\": ");
            String value = valueStart < 0 ? "" : line.substring(valueStart + 3);
            if (value.endsWith(",")) value = value.substring(0, value.length() - 1);
            switch (key) {
                case "file":
                    file = unescape(unquote(value));
                    break;
                case "line":
                case "column":
                case "message":
                    fields.put(key, value);
                    break;
                case "rule":
                    add(file, new KtlintCliError(Integer.parseInt(fields.get("line")),
                            Integer.parseInt(fields.get("column")), unescape(unquote(value)),
                            unescape(unquote(fields.get("message"))), null, false));
                    fields.clear();
                    break;
                default:
                    break;
            }
        }
    }

    private static String unquote(String literal) {
        String s = literal.startsWith("\"") ? literal.substring(1) : literal;
        return s.endsWith("\"") ? s.substring(0, s.length() - 1) : s;
    }

    /** {@code \\}, {@code \"}, {@code \b}, {@code \n}, {@code \r}, {@code \t} escapes. */
    private static String unescape(String body) {
        StringBuilder out = new StringBuilder(body.length());
        for (int i = 0; i < body.length(); i++) {
            char c = body.charAt(i);
            if (c == '\\' && i + 1 < body.length()) {
                char next = body.charAt(++i);
                switch (next) {
                    case 'b': out.append('\b'); break;
                    case 'n': out.append('\n'); break;
                    case 'r': out.append('\r'); break;
                    case 't': out.append('\t'); break;
                    default: out.append(next); break;
                }
            } else {
                out.append(c);
            }
        }
        return out.toString();
    }
}
