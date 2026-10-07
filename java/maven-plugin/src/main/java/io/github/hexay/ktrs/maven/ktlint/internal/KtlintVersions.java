package io.github.hexay.ktrs.maven.ktlint.internal;

import java.util.List;
import org.apache.maven.plugin.MojoFailureException;

/** The ktlint versions ktrs implements, and what each one means on the {@code ktrs ktlint} command line. */
public final class KtlintVersions {
    public static final String V1_8 = "1.8.0";
    public static final String V2_0 = "2.0.0-ALPHA-4";

    private static final String RULE_SET_PROVIDER_V3 = "com.pinterest.ktlint.cli.ruleset.core.api.RuleSetProviderV3";
    private static final String RULE_SET_V2_PROVIDER = "io.github.ktlint.core.cli.ruleset.core.api.RuleSetV2Provider";

    private KtlintVersions() {}

    /** The drop-in's {@code --ktlint-version} option for {@code version}; fails the build for any other version. */
    public static String cliOption(String version) throws MojoFailureException {
        if (V1_8.equals(version)) return "--ktlint-version=1.8";
        if (V2_0.equals(version)) return "--ktlint-version=2.0";
        throw new MojoFailureException("ktrs runs ktlint " + V1_8 + " or " + V2_0 + ", not ktlint " + version
                + ": set <ktlintVersion> to one of them, or use com.github.gantsign.maven:ktlint-maven-plugin for "
                + "other versions.");
    }

    /** The service interfaces a rule set JAR implements for {@code version} ({@code -R} loads these). */
    public static List<String> ruleSetInterfaces(String version) {
        return V1_8.equals(version) ? List.of(RULE_SET_PROVIDER_V3) : List.of(RULE_SET_V2_PROVIDER, RULE_SET_PROVIDER_V3);
    }

    /** The service interface a reporter JAR implements for {@code version}. */
    public static String reporterInterface(String version) {
        return V1_8.equals(version)
                ? "com.pinterest.ktlint.cli.reporter.core.api.ReporterProviderV2"
                : "io.github.ktlint.core.cli.reporter.core.api.ReporterProviderV2";
    }
}
