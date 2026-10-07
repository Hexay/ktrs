package io.github.hexay.ktrs.maven.ktlint.internal;

import io.github.hexay.ktrs.maven.ktlint.ReporterConfig;
import java.io.File;
import java.util.List;
import java.util.Set;
import org.apache.maven.artifact.Artifact;
import org.apache.maven.plugin.MojoExecutionException;
import org.apache.maven.plugin.MojoFailureException;
import org.apache.maven.plugin.logging.Log;

public final class Report extends AbstractCheckSupport {
    public Report(Log log, File basedir, String modulePackaging, List<Sources> sources, boolean android,
            Set<ReporterConfig> reporterConfig, boolean verbose, boolean enableExperimentalRules, String ktlintVersion,
            List<Artifact> pluginArtifacts) {
        super(log, basedir, modulePackaging, sources, android, reporterConfig, verbose, false, "DARK_GRAY",
                enableExperimentalRules, ktlintVersion, pluginArtifacts);
    }

    public CheckResults invoke() throws MojoExecutionException, MojoFailureException {
        ModelReporter modelReporter = new ModelReporter();
        Reporters reporters = getReporter();
        AggregatedReporter reporter = new AggregatedReporter(List.of(reporters.reporter, modelReporter));

        hasErrors(new Reporters(reporter, reporters.cliOptions));

        return new CheckResults(modelReporter.getFileCount(), modelReporter.getErrors());
    }
}
