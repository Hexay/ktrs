package io.github.hexay.ktrs.maven.ktlint.internal;

import io.github.hexay.ktrs.maven.ktlint.MavenLogReporter;
import io.github.hexay.ktrs.maven.ktlint.ReporterConfig;
import java.io.File;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Set;
import org.apache.maven.artifact.Artifact;
import org.apache.maven.plugin.MojoExecutionException;
import org.apache.maven.plugin.MojoFailureException;
import org.apache.maven.plugin.logging.Log;

public final class Check extends AbstractCheckSupport {
    private final boolean failOnViolation;

    public Check(Log log, File basedir, String modulePackaging, List<Sources> sources, boolean android,
            Set<ReporterConfig> reporterConfig, boolean verbose, boolean reporterColor, String reporterColorName,
            boolean enableExperimentalRules, boolean failOnViolation, String ktlintVersion,
            List<Artifact> pluginArtifacts) {
        super(log, basedir, modulePackaging, sources, android, addMavenReporter(reporterConfig), verbose, reporterColor,
                reporterColorName, enableExperimentalRules, ktlintVersion, pluginArtifacts);
        this.failOnViolation = failOnViolation;
    }

    public void invoke() throws MojoExecutionException, MojoFailureException {
        Reporters reporter = getReporter();

        boolean hasErrors = hasErrors(reporter);
        if (hasErrors && failOnViolation) {
            throw new MojoFailureException("Kotlin source failed ktlint check.");
        }
    }

    static Set<ReporterConfig> addMavenReporter(Set<ReporterConfig> reporterConfig) {
        if (reporterConfig.stream().anyMatch(it -> MavenLogReporter.NAME.equals(it.getName()))) {
            return reporterConfig;
        }
        Set<ReporterConfig> withMaven = new LinkedHashSet<>(reporterConfig);
        withMaven.add(new ReporterConfig(MavenLogReporter.NAME));
        return withMaven;
    }
}
