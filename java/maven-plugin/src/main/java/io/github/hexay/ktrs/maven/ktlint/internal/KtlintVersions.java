package io.github.hexay.ktrs.maven.ktlint.internal;

import static io.github.hexay.ktrs.KtlintJars.V1_8;
import static io.github.hexay.ktrs.KtlintJars.V2_0;

import org.apache.maven.plugin.MojoFailureException;

/** The ktlint versions ktrs implements, as {@code ktrs ktlint} options. */
public final class KtlintVersions {
    private KtlintVersions() {}

    /** The drop-in's {@code --ktlint-version} option for {@code version}; fails the build for any other version. */
    public static String cliOption(String version) throws MojoFailureException {
        if (V1_8.equals(version)) return "--ktlint-version=1.8";
        if (V2_0.equals(version)) return "--ktlint-version=2.0";
        throw new MojoFailureException("ktrs runs ktlint " + V1_8 + " or " + V2_0 + ", not ktlint " + version
                + ": set <ktlintVersion> to one of them, or use com.github.gantsign.maven:ktlint-maven-plugin for "
                + "other versions.");
    }
}
