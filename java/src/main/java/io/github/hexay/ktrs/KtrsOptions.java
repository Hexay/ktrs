package io.github.hexay.ktrs;

import java.io.Serializable;
import java.nio.file.Path;
import java.util.Objects;

/**
 * ktfmt's formatting options. Immutable: the {@code with} methods return a copy. Unset values take
 * the style's defaults.
 */
public final class KtrsOptions implements Serializable {
    private static final long serialVersionUID = 1L;

    public enum Style { META, GOOGLE, KOTLINLANG }

    public enum TrailingCommas { NONE, ONLY_ADD, COMPLETE }

    private final Style style;
    private final Integer maxWidth;
    private final Integer blockIndent;
    private final Integer continuationIndent;
    private final boolean removeUnusedImports;
    private final TrailingCommas trailingCommas;
    private final boolean editorConfig;

    private KtrsOptions(Style style, Integer maxWidth, Integer blockIndent, Integer continuationIndent,
            boolean removeUnusedImports, TrailingCommas trailingCommas, boolean editorConfig) {
        this.style = Objects.requireNonNull(style);
        this.maxWidth = maxWidth;
        this.blockIndent = blockIndent;
        this.continuationIndent = continuationIndent;
        this.removeUnusedImports = removeUnusedImports;
        this.trailingCommas = trailingCommas;
        this.editorConfig = editorConfig;
    }

    public static KtrsOptions of(Style style) {
        return new KtrsOptions(style, null, null, null, true, null, false);
    }

    public static KtrsOptions meta() {
        return of(Style.META);
    }

    public static KtrsOptions google() {
        return of(Style.GOOGLE);
    }

    public static KtrsOptions kotlinlang() {
        return of(Style.KOTLINLANG);
    }

    public KtrsOptions withMaxWidth(int maxWidth) {
        return new KtrsOptions(style, positive(maxWidth), blockIndent, continuationIndent, removeUnusedImports,
                trailingCommas, editorConfig);
    }

    public KtrsOptions withBlockIndent(int blockIndent) {
        return new KtrsOptions(style, maxWidth, positive(blockIndent), continuationIndent, removeUnusedImports,
                trailingCommas, editorConfig);
    }

    public KtrsOptions withContinuationIndent(int continuationIndent) {
        return new KtrsOptions(style, maxWidth, blockIndent, positive(continuationIndent), removeUnusedImports,
                trailingCommas, editorConfig);
    }

    public KtrsOptions withRemoveUnusedImports(boolean removeUnusedImports) {
        return new KtrsOptions(style, maxWidth, blockIndent, continuationIndent, removeUnusedImports,
                trailingCommas, editorConfig);
    }

    public KtrsOptions withTrailingCommas(TrailingCommas trailingCommas) {
        return new KtrsOptions(style, maxWidth, blockIndent, continuationIndent, removeUnusedImports,
                Objects.requireNonNull(trailingCommas), editorConfig);
    }

    /** Applies the {@code .editorconfig} files that govern the formatted file's path, over these options. */
    public KtrsOptions withEditorConfig(boolean editorConfig) {
        return new KtrsOptions(style, maxWidth, blockIndent, continuationIndent, removeUnusedImports,
                trailingCommas, editorConfig);
    }

    /** The request header for {@code ktrs serve} (protocol: crates/ktrs-cli/src/serve.rs). */
    String header(Path file) {
        StringBuilder header = new StringBuilder();
        line(header, "style", style.name().toLowerCase());
        line(header, "max-width", maxWidth);
        line(header, "block-indent", blockIndent);
        line(header, "continuation-indent", continuationIndent);
        line(header, "remove-unused-imports", removeUnusedImports);
        line(header, "trailing-commas", trailingCommas == null ? null : trailingCommas.name().toLowerCase());
        line(header, "editorconfig", editorConfig);
        line(header, "path", file);
        return header.toString();
    }

    private static void line(StringBuilder header, String key, Object value) {
        if (value == null) {
            return;
        }
        String text = value.toString();
        if (text.indexOf('\n') >= 0 || text.indexOf('\r') >= 0) {
            throw new IllegalArgumentException(key + " contains a line break: " + text);
        }
        header.append(key).append('=').append(text).append('\n');
    }

    private static int positive(int value) {
        if (value <= 0) {
            throw new IllegalArgumentException("expected a positive value, got " + value);
        }
        return value;
    }

    @Override
    public boolean equals(Object o) {
        if (!(o instanceof KtrsOptions)) {
            return false;
        }
        KtrsOptions other = (KtrsOptions) o;
        return style == other.style && Objects.equals(maxWidth, other.maxWidth)
                && Objects.equals(blockIndent, other.blockIndent)
                && Objects.equals(continuationIndent, other.continuationIndent)
                && removeUnusedImports == other.removeUnusedImports && trailingCommas == other.trailingCommas
                && editorConfig == other.editorConfig;
    }

    @Override
    public int hashCode() {
        return Objects.hash(style, maxWidth, blockIndent, continuationIndent, removeUnusedImports, trailingCommas,
                editorConfig);
    }

    @Override
    public String toString() {
        return "KtrsOptions{" + header(null).trim().replace('\n', ',') + "}";
    }
}
