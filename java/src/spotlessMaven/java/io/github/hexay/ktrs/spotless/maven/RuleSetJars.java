package io.github.hexay.ktrs.spotless.maven;

import java.io.ByteArrayOutputStream;
import java.io.File;
import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.io.UncheckedIOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.StandardCopyOption;
import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.util.ArrayList;
import java.util.Collection;
import java.util.Collections;
import java.util.HashSet;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.zip.ZipEntry;
import java.util.zip.ZipFile;
import java.util.zip.ZipOutputStream;

/**
 * Turns Spotless's resolved {@code customRuleSets} (the GAVs with their dependencies, as on stock ktlint's classpath)
 * into the rule set JARs {@code ktrs serve} takes, as the Gradle plugin's {@code JarServices} does: ktrs loads each JAR
 * on its own, so compose-rules' Maven artifact and its {@code common-ktlint} dependency become one merged JAR.
 */
final class RuleSetJars {
    private static final String SERVICES = "META-INF/services/";
    private static final List<String> PROVIDER_SERVICES = List.of(
            SERVICES + "com.pinterest.ktlint.cli.ruleset.core.api.RuleSetProviderV3",
            SERVICES + "io.github.ktlint.core.cli.ruleset.core.api.RuleSetV2Provider");
    private static final String COMPOSE_RULES = "io/nlopez/compose/";

    private RuleSetJars() {}

    /**
     * The JARs among {@code resolved} that declare a rule set provider or hold compose-rules classes: the JAR itself
     * when it is the only one, else all of them merged into a JAR in {@code dir}.
     */
    static List<File> of(Collection<String> coordinates, Collection<File> resolved, File dir) {
        List<File> jars = new ArrayList<>();
        for (File file : resolved) {
            if (file.getName().endsWith(".jar") && isRuleSetJar(file)) jars.add(file);
        }
        if (jars.isEmpty()) {
            throw new IllegalArgumentException("customRuleSets " + coordinates + " declare no ktlint rule set provider");
        }
        if (jars.size() == 1) return jars;
        File merged = new File(dir, "ktlint-rulesets-" + key(jars) + ".jar");
        if (!merged.isFile()) merge(jars, merged);
        return List.of(merged);
    }

    private static boolean isRuleSetJar(File jar) {
        try (ZipFile zip = new ZipFile(jar)) {
            return PROVIDER_SERVICES.stream().anyMatch(name -> zip.getEntry(name) != null)
                    || zip.stream().anyMatch(entry -> entry.getName().startsWith(COMPOSE_RULES));
        } catch (IOException e) {
            throw new UncheckedIOException("reading rule set JAR " + jar, e);
        }
    }

    /** Names the merged JAR after its inputs, so an unchanged set reuses it and Spotless's up-to-date check holds. */
    private static String key(List<File> jars) {
        try {
            MessageDigest digest = MessageDigest.getInstance("SHA-256");
            for (File jar : jars) {
                digest.update((jar.getAbsolutePath() + '\t' + jar.length() + '\t' + jar.lastModified() + '\n')
                        .getBytes(StandardCharsets.UTF_8));
            }
            StringBuilder hex = new StringBuilder();
            for (byte b : digest.digest()) hex.append(String.format("%02x", b));
            return hex.substring(0, 16);
        } catch (NoSuchAlgorithmException e) {
            throw new IllegalStateException(e);
        }
    }

    /** The first of duplicate entries wins and service files are concatenated, as in {@code JarServices.merge}. */
    private static void merge(List<File> jars, File target) {
        try {
            Files.createDirectories(target.getParentFile().toPath());
            File partial = Files.createTempFile(target.getParentFile().toPath(), target.getName(), ".partial").toFile();
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
            // Parallel Maven builds can merge the same set concurrently: whoever moves last wins, with equal content.
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
        return name.startsWith("META-INF/")
                && (name.endsWith(".SF") || name.endsWith(".DSA") || name.endsWith(".RSA") || name.endsWith(".EC"));
    }
}
