package io.github.hexay.ktrs.maven.ktlint;

import io.github.hexay.ktrs.maven.ktlint.internal.Sources;
import java.io.File;
import java.util.List;
import java.util.Set;
import org.apache.maven.plugin.AbstractMojo;
import org.apache.maven.plugin.descriptor.PluginDescriptor;
import org.apache.maven.plugins.annotations.Parameter;

public abstract class AbstractBaseMojo extends AbstractMojo {
    @Parameter(defaultValue = "${project.basedir}", readonly = true, required = true)
    protected File basedir;

    @Parameter(defaultValue = "${project.packaging}", readonly = true, required = true)
    protected String packaging;

    @Parameter(defaultValue = "${project.compileSourceRoots}", readonly = true, required = true)
    protected List<String> sourceRoots;

    @Parameter(defaultValue = "${project.testCompileSourceRoots}", readonly = true, required = true)
    protected List<String> testSourceRoots;

    /**
     * A list of root directories containing Kotlin scripts.
     */
    @Parameter(property = "ktlint.scriptRoots", defaultValue = "${project.basedir.path}")
    protected List<String> scriptRoots;

    /**
     * Include the production source roots.
     */
    @Parameter(property = "ktlint.includeSources", defaultValue = "true", required = true)
    protected boolean includeSources = true;

    /**
     * Include the test source roots.
     */
    @Parameter(property = "ktlint.includeTestSources", defaultValue = "true", required = true)
    protected boolean includeTestSources = true;

    /**
     * Include scripts.
     */
    @Parameter(property = "ktlint.includeScripts", defaultValue = "true", required = true)
    protected boolean includeScripts = true;

    /**
     * A list of inclusion filters for the source files to be processed under the source roots.
     */
    @Parameter(defaultValue = "**/*.kt")
    protected Set<String> sourcesIncludes;

    /**
     * A list of exclusion filters for the source files to be processed under the source roots.
     */
    @Parameter
    protected Set<String> sourcesExcludes;

    /**
     * A list of inclusion filters for the source files to be processed under the test source roots.
     */
    @Parameter(defaultValue = "**/*.kt")
    protected Set<String> testSourcesIncludes;

    /**
     * A list of exclusion filters for the source files to be processed under the test source roots.
     */
    @Parameter
    protected Set<String> testSourcesExcludes;

    /**
     * A list of inclusion filters for scripts.
     */
    @Parameter(defaultValue = "*.kts")
    protected Set<String> scriptsIncludes;

    /**
     * A list of exclusion filters for scripts.
     */
    @Parameter
    protected Set<String> scriptsExcludes;

    /**
     * Enable Android Kotlin Style Guide compatibility.
     */
    @Parameter(property = "ktlint.android", defaultValue = "false", required = true)
    protected boolean android = false;

    /**
     * Enable experimental rules (ktlint-ruleset-experimental).
     */
    @Parameter(property = "ktlint.experimental", defaultValue = "false", required = true)
    protected boolean experimental = false;

    /**
     * The ktlint version ktrs behaves as: {@code 1.8.0} (gantsign 3.7.1's) or {@code 2.0.0-ALPHA-4}.
     */
    @Parameter(property = "ktrs.ktlintVersion", defaultValue = "1.8.0", required = true)
    protected String ktlintVersion;

    /** The plugin's resolved dependencies: the rule set and reporter JARs users add as plugin dependencies. */
    @Parameter(defaultValue = "${plugin}", readonly = true, required = true)
    protected PluginDescriptor plugin;

    /** The source, test source and script roots, as gantsign's goals list them. */
    protected List<Sources> sources() {
        return List.of(
                new Sources(includeSources, sourceRoots, sourcesIncludes, sourcesExcludes),
                new Sources(includeTestSources, testSourceRoots, testSourcesIncludes, testSourcesExcludes),
                new Sources(includeScripts, scriptRoots, scriptsIncludes, scriptsExcludes));
    }
}
