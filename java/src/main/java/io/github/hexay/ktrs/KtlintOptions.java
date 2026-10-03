package io.github.hexay.ktrs;

import java.io.File;
import java.io.Serializable;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.Collection;
import java.util.Collections;
import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.SortedMap;
import java.util.TreeMap;

/**
 * The options of Spotless's {@code ktlint()} step, for {@link Ktrs#ktlint}. Immutable: the {@code with}
 * methods return a copy.
 */
public final class KtlintOptions implements Serializable {
    private static final long serialVersionUID = 1L;

    public static final String DEFAULT_VERSION = "1.8.0";
    /** ktlint 2.0's pre-release, which ktrs's 2.0 mode matches. */
    public static final String VERSION_2_0 = "2.0.0-ALPHA-4";

    private final String version;
    private final SortedMap<String, String> editorConfigOverride;
    private final File editorConfigPath;
    private final List<File> customRuleSets;

    private KtlintOptions(String version, SortedMap<String, String> editorConfigOverride, File editorConfigPath,
            List<File> customRuleSets) {
        this.version = version;
        this.editorConfigOverride = Collections.unmodifiableSortedMap(editorConfigOverride);
        this.editorConfigPath = editorConfigPath;
        this.customRuleSets = Collections.unmodifiableList(customRuleSets);
    }

    /** ktlint {@value #DEFAULT_VERSION}. */
    public static KtlintOptions defaults() {
        return of(DEFAULT_VERSION);
    }

    /** @throws IllegalArgumentException unless {@code version} is {@value #DEFAULT_VERSION} or {@value #VERSION_2_0} */
    public static KtlintOptions of(String version) {
        if (!DEFAULT_VERSION.equals(version) && !VERSION_2_0.equals(version)) {
            throw new IllegalArgumentException("ktrs matches ktlint " + DEFAULT_VERSION + " and " + VERSION_2_0
                    + ", not " + version);
        }
        return new KtlintOptions(version, new TreeMap<>(), null, new ArrayList<>());
    }

    /**
     * {@code .editorconfig} properties that win over the files (Spotless's {@code editorConfigOverride}): values are
     * used as their {@code toString()}. As in Spotless, unknown properties are ignored, and unless the map or the
     * {@linkplain #withEditorConfigPath editorconfig path} sets {@code ktlint_code_style}, a non-empty map also sets
     * it to {@code intellij_idea}.
     */
    public KtlintOptions withEditorConfigOverride(Map<String, ?> editorConfigOverride) {
        SortedMap<String, String> values = new TreeMap<>();
        editorConfigOverride.forEach((key, value) -> values.put(Objects.requireNonNull(key),
                Objects.requireNonNull(value, () -> "editorConfigOverride value of " + key).toString()));
        return new KtlintOptions(version, values, editorConfigPath, new ArrayList<>(customRuleSets));
    }

    /**
     * An {@code .editorconfig}-format file whose values apply where no {@code .editorconfig} on a file's path sets
     * them (Spotless's {@code setEditorConfigPath}; Spotless's Gradle plugin defaults it to the root project's
     * {@code .editorconfig}). Null for none.
     */
    public KtlintOptions withEditorConfigPath(File editorConfigPath) {
        return new KtlintOptions(version, new TreeMap<>(editorConfigOverride), editorConfigPath,
                new ArrayList<>(customRuleSets));
    }

    /**
     * Rule set JARs (Spotless's {@code customRuleSets}, as files). ktrs runs the compose-rules release it ports
     * natively; any other JAR fails the request.
     */
    public KtlintOptions withCustomRuleSets(Collection<File> customRuleSets) {
        List<File> files = new ArrayList<>(customRuleSets);
        files.forEach(Objects::requireNonNull);
        return new KtlintOptions(version, new TreeMap<>(editorConfigOverride), editorConfigPath, files);
    }

    public String version() {
        return version;
    }

    public File editorConfigPath() {
        return editorConfigPath;
    }

    public List<File> customRuleSets() {
        return customRuleSets;
    }

    /** The request header for {@code ktrs serve} (protocol: crates/ktrs-cli/src/serve_ktlint.rs). */
    String header(Path file) {
        RequestHeader header = new RequestHeader()
                .line("tool", "ktlint")
                .line("ktlint-version", version.equals(VERSION_2_0) ? "2.0" : "1.8")
                .line("editorconfig-defaults", editorConfigPath == null ? null : editorConfigPath.getAbsolutePath());
        editorConfigOverride.forEach((key, value) -> header.line("editorconfig-override", key + "=" + value));
        customRuleSets.forEach(jar -> header.line("ruleset", jar.getAbsolutePath()));
        return header.line("path", file).toString();
    }

    @Override
    public boolean equals(Object o) {
        if (!(o instanceof KtlintOptions)) {
            return false;
        }
        KtlintOptions other = (KtlintOptions) o;
        return version.equals(other.version) && editorConfigOverride.equals(other.editorConfigOverride)
                && Objects.equals(editorConfigPath, other.editorConfigPath)
                && customRuleSets.equals(other.customRuleSets);
    }

    @Override
    public int hashCode() {
        return Objects.hash(version, editorConfigOverride, editorConfigPath, customRuleSets);
    }

    @Override
    public String toString() {
        return "KtlintOptions{" + header(null).trim().replace('\n', ',') + "}";
    }
}
