package io.github.hexay.ktrs.maven.ktlint.internal;

import io.github.hexay.ktrs.KtlintJars;
import java.io.File;
import java.io.IOException;
import java.io.InputStream;
import java.io.UncheckedIOException;
import java.nio.charset.StandardCharsets;
import java.util.HashSet;
import java.util.List;
import java.util.Set;
import java.util.stream.Collectors;
import org.apache.maven.artifact.Artifact;

/** The plugin {@code <dependencies>} gantsign loads with {@code ServiceLoader}; turned into JARs by {@link KtlintJars}. */
public final class JarServices {
    private static final List<String> KTLINT_GROUPS = List.of("com.pinterest.ktlint", "io.github.ktlint.core");
    private static final String KTLINT_REPORTER_PREFIX = "ktlint-cli-reporter-";

    private JarServices() {}

    /** The JARs among the plugin's {@code artifacts} that users added, without ktlint and its runtime. */
    public static List<File> userJars(List<Artifact> artifacts) {
        Set<String> own = pluginArtifacts();
        return artifacts.stream()
                .filter(a -> !own.contains(a.getGroupId() + ":" + a.getArtifactId()))
                .filter(a -> !KtlintJars.isKtlintRuntime(a.getGroupId()))
                .map(Artifact::getFile)
                .filter(f -> f != null && f.isFile() && f.getName().endsWith(".jar"))
                .collect(Collectors.toList());
    }

    /** Ids of ktlint's own reporters added as plugin dependencies ({@code ktlint-cli-reporter-html}: {@code html}). */
    public static List<String> ktlintReporterIds(List<Artifact> artifacts) {
        return artifacts.stream()
                .filter(a -> KTLINT_GROUPS.contains(a.getGroupId()) && a.getArtifactId().startsWith(KTLINT_REPORTER_PREFIX))
                .map(a -> a.getArtifactId().substring(KTLINT_REPORTER_PREFIX.length()))
                .filter(id -> !id.equals("core"))
                .collect(Collectors.toList());
    }

    private static Set<String> pluginArtifacts() {
        try (InputStream in = JarServices.class.getResourceAsStream("plugin-artifacts.txt")) {
            if (in == null) return Set.of();
            return new HashSet<>(List.of(new String(in.readAllBytes(), StandardCharsets.UTF_8).split("\\R")));
        } catch (IOException e) {
            throw new UncheckedIOException(e);
        }
    }
}
