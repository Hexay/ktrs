package io.github.hexay.ktrs.spotless.maven;

import io.github.hexay.ktrs.KtlintJars;
import java.io.File;
import java.io.IOException;
import java.io.UncheckedIOException;
import java.nio.charset.StandardCharsets;
import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.util.ArrayList;
import java.util.Collection;
import java.util.List;
import java.util.zip.ZipFile;

/**
 * Turns Spotless's resolved {@code customRuleSets} (the GAVs with their dependencies, as on stock ktlint's classpath)
 * into the rule set JARs {@code ktrs serve} takes; see {@link KtlintJars}.
 */
final class RuleSetJars {
    private static final List<String> PROVIDERS = KtlintJars.ruleSetInterfaces(KtlintJars.V2_0);
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
        if (!merged.isFile()) KtlintJars.merge(jars, merged);
        return List.of(merged);
    }

    private static boolean isRuleSetJar(File jar) {
        if (PROVIDERS.stream().anyMatch(it -> KtlintJars.declaresService(jar, it))) return true;
        try (ZipFile zip = new ZipFile(jar)) {
            return zip.stream().anyMatch(entry -> entry.getName().startsWith(COMPOSE_RULES));
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
}
