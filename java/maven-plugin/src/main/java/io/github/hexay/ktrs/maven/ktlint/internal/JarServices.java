package io.github.hexay.ktrs.maven.ktlint.internal;

import java.io.ByteArrayOutputStream;
import java.io.File;
import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.io.UncheckedIOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.util.Enumeration;
import java.util.HashSet;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.stream.Collectors;
import java.util.zip.ZipEntry;
import java.util.zip.ZipFile;
import java.util.zip.ZipOutputStream;
import org.apache.maven.artifact.Artifact;

/**
 * The plugin {@code <dependencies>} as ktlint CLI {@code -R} / {@code artifact=} JARs. gantsign loads them with
 * {@code ServiceLoader} from the plugin realm; the CLI loads each JAR on its own and rejects one that declares no
 * provider, so a rule set with dependencies (compose-rules' Maven artifact needs {@code common-ktlint}) becomes one
 * merged JAR. Same rules as the Gradle plugin's {@code JarServices.kt}.
 */
public final class JarServices {
    // What the ktlint jar itself provides: loading them again would shadow its own classes.
    private static final List<String> PROVIDED_GROUPS = List.of("com.pinterest", "io.github.ktlint", "org.jetbrains",
            "io.github.oshai", "org.slf4j", "org.ec4j", "ch.qos.logback");
    private static final List<String> KTLINT_GROUPS = List.of("com.pinterest.ktlint", "io.github.ktlint.core");
    private static final String KTLINT_REPORTER_PREFIX = "ktlint-cli-reporter-";
    private static final String SERVICES = "META-INF/services/";
    private static final List<String> SIGNATURE_SUFFIXES = List.of(".SF", ".DSA", ".RSA", ".EC");

    private JarServices() {}

    /** The JARs among the plugin's {@code artifacts} that users added, without ktlint and its runtime. */
    public static List<File> userJars(List<Artifact> artifacts) {
        Set<String> own = pluginArtifacts();
        return artifacts.stream()
                .filter(a -> !own.contains(a.getGroupId() + ":" + a.getArtifactId()))
                .filter(a -> PROVIDED_GROUPS.stream().noneMatch(g -> a.getGroupId().equals(g) || a.getGroupId().startsWith(g + ".")))
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

    /**
     * The {@code -R} JARs for {@code jars} and ktlint {@code version}: none when no JAR declares a rule set
     * provider, the JAR itself when it is the only one (so ktrs recognizes a release it runs natively), else all of
     * them merged into one JAR in {@code tempDir}.
     */
    public static List<File> ruleSetJars(List<File> jars, String version, File tempDir) {
        List<String> interfaces = KtlintVersions.ruleSetInterfaces(version);
        if (jars.stream().noneMatch(jar -> interfaces.stream().anyMatch(it -> declaresService(jar, it)))) return List.of();
        if (jars.size() == 1) return jars;
        return List.of(merge(jars, new File(tempDir, "ktlint-rulesets.jar")));
    }

    /** The reporter JARs among {@code jars} for ktlint {@code version}. */
    public static List<File> reporterJars(List<File> jars, String version) {
        String reporterInterface = KtlintVersions.reporterInterface(version);
        return jars.stream().filter(it -> declaresService(it, reporterInterface)).collect(Collectors.toList());
    }

    private static Set<String> pluginArtifacts() {
        try (InputStream in = JarServices.class.getResourceAsStream("plugin-artifacts.txt")) {
            if (in == null) return Set.of();
            return new HashSet<>(List.of(new String(in.readAllBytes(), StandardCharsets.UTF_8).split("\\R")));
        } catch (IOException e) {
            throw new UncheckedIOException(e);
        }
    }

    private static boolean declaresService(File jar, String serviceInterface) {
        try (ZipFile zip = new ZipFile(jar)) {
            return zip.getEntry(SERVICES + serviceInterface) != null;
        } catch (IOException e) {
            return false;
        }
    }

    /** {@code jars}' entries in one JAR: the first of duplicate entries wins, service files are concatenated. */
    private static File merge(List<File> jars, File target) {
        target.getParentFile().mkdirs();
        Map<String, ByteArrayOutputStream> services = new LinkedHashMap<>();
        Set<String> written = new HashSet<>();
        try (ZipOutputStream out = new ZipOutputStream(Files.newOutputStream(target.toPath()))) {
            for (File jar : jars) {
                try (ZipFile zip = new ZipFile(jar)) {
                    for (Enumeration<? extends ZipEntry> e = zip.entries(); e.hasMoreElements(); ) {
                        ZipEntry entry = e.nextElement();
                        String name = entry.getName();
                        if (entry.isDirectory() || isSignature(name)) continue;
                        if (name.startsWith(SERVICES)) {
                            ByteArrayOutputStream content = services.computeIfAbsent(name, k -> new ByteArrayOutputStream());
                            copy(zip, entry, content);
                            content.write('\n');
                        } else if (written.add(name)) {
                            out.putNextEntry(new ZipEntry(name));
                            copy(zip, entry, out);
                            out.closeEntry();
                        }
                    }
                }
            }
            for (Map.Entry<String, ByteArrayOutputStream> service : services.entrySet()) {
                out.putNextEntry(new ZipEntry(service.getKey()));
                out.write(service.getValue().toByteArray());
                out.closeEntry();
            }
        } catch (IOException e) {
            throw new UncheckedIOException(e);
        }
        return target;
    }

    private static void copy(ZipFile zip, ZipEntry entry, OutputStream out) throws IOException {
        try (InputStream in = zip.getInputStream(entry)) {
            in.transferTo(out);
        }
    }

    private static boolean isSignature(String name) {
        return name.startsWith("META-INF/") && SIGNATURE_SUFFIXES.stream().anyMatch(name::endsWith);
    }
}
