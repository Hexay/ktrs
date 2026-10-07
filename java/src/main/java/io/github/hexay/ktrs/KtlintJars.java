package io.github.hexay.ktrs;

import java.io.ByteArrayOutputStream;
import java.io.File;
import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.io.UncheckedIOException;
import java.nio.file.Files;
import java.nio.file.StandardCopyOption;
import java.util.Collections;
import java.util.HashSet;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.stream.Collectors;
import java.util.zip.ZipEntry;
import java.util.zip.ZipFile;
import java.util.zip.ZipOutputStream;

/**
 * Rule set and reporter JARs for {@code ktrs ktlint} ({@code -R}, reporter {@code artifact=}), shared by the ktrs build
 * plugins. Upstream plugins put the whole resolved classpath on one class loader; the ktlint CLI loads each JAR on its own
 * and rejects one that declares no provider, so a rule set with dependencies (compose-rules' Maven artifact needs
 * {@code common-ktlint}) becomes one merged JAR.
 */
public final class KtlintJars {
    public static final String V1_8 = "1.8.0";
    public static final String V2_0 = "2.0.0-ALPHA-4";

    // What the ktlint jar itself provides: loading them again would shadow its own classes.
    private static final List<String> KTLINT_RUNTIME_GROUPS = List.of("com.pinterest", "io.github.ktlint",
            "org.jetbrains", "io.github.oshai", "org.slf4j", "org.ec4j", "ch.qos.logback");
    private static final String RULE_SET_PROVIDER_V3 = "com.pinterest.ktlint.cli.ruleset.core.api.RuleSetProviderV3";
    private static final String RULE_SET_V2_PROVIDER = "io.github.ktlint.core.cli.ruleset.core.api.RuleSetV2Provider";
    private static final String SERVICES = "META-INF/services/";
    private static final List<String> SIGNATURE_SUFFIXES = List.of(".SF", ".DSA", ".RSA", ".EC");

    private KtlintJars() {}

    /** Whether Maven/Gradle {@code group} is ktlint or its runtime (Kotlin, logging, ec4j). */
    public static boolean isKtlintRuntime(String group) {
        return KTLINT_RUNTIME_GROUPS.stream().anyMatch(g -> group.equals(g) || group.startsWith(g + "."));
    }

    /** The service interfaces a rule set JAR implements for ktlint {@code version} ({@code -R} loads these). */
    public static List<String> ruleSetInterfaces(String version) {
        return V1_8.equals(version) ? List.of(RULE_SET_PROVIDER_V3) : List.of(RULE_SET_V2_PROVIDER, RULE_SET_PROVIDER_V3);
    }

    /** The service interface a reporter JAR implements for ktlint {@code version}. */
    public static String reporterInterface(String version) {
        return V1_8.equals(version)
                ? "com.pinterest.ktlint.cli.reporter.core.api.ReporterProviderV2"
                : "io.github.ktlint.core.cli.reporter.core.api.ReporterProviderV2";
    }

    /**
     * The {@code -R} JARs for {@code files} and ktlint {@code version}: none when no JAR declares a rule set provider,
     * the JAR itself when it is the only one (so ktrs recognizes a release it runs natively), else all of them merged
     * into {@code tempDir/ktlint-rulesets.jar}.
     */
    public static List<File> ruleSetJars(List<File> files, String version, File tempDir) {
        List<File> jars = files.stream().filter(f -> f.isFile() && f.getName().endsWith(".jar")).collect(Collectors.toList());
        List<String> interfaces = ruleSetInterfaces(version);
        if (jars.stream().noneMatch(jar -> interfaces.stream().anyMatch(it -> declaresService(jar, it)))) return List.of();
        if (jars.size() == 1) return jars;
        File merged = new File(tempDir, "ktlint-rulesets.jar");
        merge(jars, merged);
        return List.of(merged);
    }

    /** The reporter JARs among {@code files} for ktlint {@code version}. */
    public static List<File> reporterJars(List<File> files, String version) {
        String reporterInterface = reporterInterface(version);
        return files.stream().filter(f -> f.isFile() && declaresService(f, reporterInterface)).collect(Collectors.toList());
    }

    public static boolean declaresService(File jar, String serviceInterface) {
        try (ZipFile zip = new ZipFile(jar)) {
            return zip.getEntry(SERVICES + serviceInterface) != null;
        } catch (IOException e) {
            return false;
        }
    }

    /**
     * Writes {@code jars}' entries to {@code target}: the first of duplicate entries wins, service files are
     * concatenated, signatures dropped. Atomic, so parallel builds merging the same set into one path are safe.
     */
    public static void merge(List<File> jars, File target) {
        try {
            Files.createDirectories(target.getAbsoluteFile().getParentFile().toPath());
            File partial = Files.createTempFile(target.getAbsoluteFile().getParentFile().toPath(), target.getName(), ".partial").toFile();
            Map<String, ByteArrayOutputStream> services = new LinkedHashMap<>();
            Set<String> written = new HashSet<>();
            try (ZipOutputStream out = new ZipOutputStream(Files.newOutputStream(partial.toPath()))) {
                for (File jar : jars) {
                    try (ZipFile zip = new ZipFile(jar)) {
                        for (ZipEntry entry : Collections.list(zip.entries())) {
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
                    service.getValue().writeTo(out);
                    out.closeEntry();
                }
            }
            Files.move(partial.toPath(), target.toPath(), StandardCopyOption.REPLACE_EXISTING);
        } catch (IOException e) {
            throw new UncheckedIOException("merging rule set JARs " + jars + " into " + target, e);
        }
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
