package io.github.hexay.ktrs.maven.ktlint;

import java.util.ArrayList;
import java.util.List;
import java.util.stream.Collectors;
import org.apache.maven.plugin.logging.Log;

/** A Maven {@link Log} that keeps {@code [LEVEL] message} lines; debug is enabled. */
public final class RecordingLog implements Log {
    public final List<String> lines = new ArrayList<>();

    public List<String> at(String level) {
        String prefix = "[" + level + "] ";
        return lines.stream().filter(l -> l.startsWith(prefix)).map(l -> l.substring(prefix.length()))
                .collect(Collectors.toList());
    }

    private void add(String level, CharSequence content) {
        lines.add("[" + level + "] " + content);
    }

    @Override public boolean isDebugEnabled() { return true; }
    @Override public void debug(CharSequence content) { add("DEBUG", content); }
    @Override public void debug(CharSequence content, Throwable error) { add("DEBUG", content); }
    @Override public void debug(Throwable error) { add("DEBUG", error.toString()); }
    @Override public boolean isInfoEnabled() { return true; }
    @Override public void info(CharSequence content) { add("INFO", content); }
    @Override public void info(CharSequence content, Throwable error) { add("INFO", content); }
    @Override public void info(Throwable error) { add("INFO", error.toString()); }
    @Override public boolean isWarnEnabled() { return true; }
    @Override public void warn(CharSequence content) { add("WARNING", content); }
    @Override public void warn(CharSequence content, Throwable error) { add("WARNING", content); }
    @Override public void warn(Throwable error) { add("WARNING", error.toString()); }
    @Override public boolean isErrorEnabled() { return true; }
    @Override public void error(CharSequence content) { add("ERROR", content); }
    @Override public void error(CharSequence content, Throwable error) { add("ERROR", content); }
    @Override public void error(Throwable error) { add("ERROR", error.toString()); }
}
