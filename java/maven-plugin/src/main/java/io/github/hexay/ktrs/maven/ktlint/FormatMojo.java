package io.github.hexay.ktrs.maven.ktlint;

import io.github.hexay.ktrs.maven.ktlint.internal.Format;
import org.apache.maven.plugin.MojoExecutionException;
import org.apache.maven.plugin.MojoFailureException;
import org.apache.maven.plugins.annotations.LifecyclePhase;
import org.apache.maven.plugins.annotations.Mojo;
import org.apache.maven.plugins.annotations.Parameter;

/**
 * Automatically fixes violations of the code style (when possible).
 */
@Mojo(name = "format", defaultPhase = LifecyclePhase.PROCESS_SOURCES, requiresProject = true, threadSafe = true)
public class FormatMojo extends AbstractBaseMojo {
    /**
     * Skips automatic code style fixes.
     */
    @Parameter(property = "ktlint.skip", defaultValue = "false", required = true)
    private boolean skip = false;

    @Override
    public void execute() throws MojoExecutionException, MojoFailureException {
        if (skip) {
            return;
        }
        new Format(getLog(), basedir, packaging, sources(), android, experimental, ktlintVersion,
                plugin.getArtifacts()).invoke();
    }
}
