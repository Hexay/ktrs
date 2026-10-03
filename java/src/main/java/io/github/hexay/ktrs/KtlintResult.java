package io.github.hexay.ktrs;

import java.util.ArrayList;
import java.util.Collections;
import java.util.List;
import java.util.Objects;

/** What {@link Ktrs#ktlint} returns: the formatted code and the violations it could not autocorrect. */
public final class KtlintResult {
    private final String code;
    private final boolean changed;
    private final List<Violation> violations;

    KtlintResult(String code, boolean changed, List<Violation> violations) {
        this.code = code;
        this.changed = changed;
        this.violations = Collections.unmodifiableList(violations);
    }

    public String code() {
        return code;
    }

    public boolean changed() {
        return changed;
    }

    /** Sorted by position. */
    public List<Violation> violations() {
        return violations;
    }

    /** A violation ktlint could not autocorrect. */
    public static final class Violation {
        private final int line;
        private final int col;
        private final String ruleId;
        private final String detail;

        public Violation(int line, int col, String ruleId, String detail) {
            this.line = line;
            this.col = col;
            this.ruleId = Objects.requireNonNull(ruleId);
            this.detail = Objects.requireNonNull(detail);
        }

        public int line() {
            return line;
        }

        public int col() {
            return col;
        }

        /** {@code <rule set>:<rule>}, e.g. {@code standard:no-wildcard-imports}. */
        public String ruleId() {
            return ruleId;
        }

        public String detail() {
            return detail;
        }

        @Override
        public boolean equals(Object o) {
            if (!(o instanceof Violation)) {
                return false;
            }
            Violation other = (Violation) o;
            return line == other.line && col == other.col && ruleId.equals(other.ruleId) && detail.equals(other.detail);
        }

        @Override
        public int hashCode() {
            return Objects.hash(line, col, ruleId, detail);
        }

        @Override
        public String toString() {
            return line + ":" + col + " " + detail + " (" + ruleId + ")";
        }
    }

    /** Parses the response's {@code violation=} header lines (protocol: crates/ktrs-cli/src/serve_ktlint.rs). */
    static KtlintResult parse(String code, boolean changed, List<String> header) {
        List<Violation> violations = new ArrayList<>();
        for (String line : header) {
            if (!line.startsWith("violation=")) {
                continue;
            }
            String[] fields = line.substring("violation=".length()).split("\t", 4);
            if (fields.length != 4) {
                throw new KtrsException("malformed violation from ktrs serve: " + line);
            }
            violations.add(new Violation(Integer.parseInt(fields[0]), Integer.parseInt(fields[1]), fields[2],
                    unescape(fields[3])));
        }
        return new KtlintResult(code, changed, violations);
    }

    private static String unescape(String text) {
        StringBuilder out = new StringBuilder(text.length());
        for (int i = 0; i < text.length(); i++) {
            char c = text.charAt(i);
            if (c != '\\' || i + 1 == text.length()) {
                out.append(c);
                continue;
            }
            char escaped = text.charAt(++i);
            out.append(escaped == 'n' ? '\n' : escaped == 'r' ? '\r' : escaped == 't' ? '\t' : escaped);
        }
        return out.toString();
    }
}
