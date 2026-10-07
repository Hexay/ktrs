package io.github.hexay.ktrs.maven.ktlint;

import io.github.hexay.ktrs.maven.ktlint.internal.Check;
import java.util.Set;
import org.apache.maven.plugin.MojoExecutionException;
import org.apache.maven.plugin.MojoFailureException;
import org.apache.maven.plugins.annotations.LifecyclePhase;
import org.apache.maven.plugins.annotations.Mojo;
import org.apache.maven.plugins.annotations.Parameter;

/**
 * Checks for violations of the code style.
 */
@Mojo(name = "check", defaultPhase = LifecyclePhase.VERIFY, requiresProject = true, threadSafe = true)
public class CheckMojo extends AbstractBaseMojo {
    /**
     * A set of reporters to output the results to.
     */
    @Parameter
    private Set<ReporterConfig> reporters;

    /**
     * Whether the KtLint reporter should output in color (doesn't affect the Maven output).
     */
    @Parameter
    private boolean reporterColor = false;

    /**
     * The color the KtLint reporter should output in (doesn't affect the Maven output).
     */
    @Parameter
    private String reporterColorName = "DARK_GRAY";

    /**
     * Show error codes.
     */
    @Parameter(property = "ktlint.verbose", defaultValue = "false", required = true)
    private boolean verbose = false;

    /**
     * Whether to fail the build if the linter finds violations of the code style.
     */
    @Parameter(property = "ktlint.failOnViolation", defaultValue = "true", required = true)
    private boolean failOnViolation = true;

    /**
     * Skips and code style checks.
     */
    @Parameter(property = "ktlint.skip", defaultValue = "false", required = true)
    private boolean skip = false;

    @Override
    public void execute() throws MojoExecutionException, MojoFailureException {
        if (skip) {
            return;
        }
        new Check(getLog(), basedir, packaging, sources(), android, reporters == null ? Set.of() : reporters, verbose,
                reporterColor, reporterColorName, experimental, failOnViolation, ktlintVersion,
                plugin.getArtifacts()).invoke();
    }
}
