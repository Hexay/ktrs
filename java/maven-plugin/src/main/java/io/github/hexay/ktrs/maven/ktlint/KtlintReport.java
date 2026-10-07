package io.github.hexay.ktrs.maven.ktlint;

import io.github.hexay.ktrs.maven.ktlint.internal.CheckResults;
import io.github.hexay.ktrs.maven.ktlint.internal.KtlintReportGenerator;
import io.github.hexay.ktrs.maven.ktlint.internal.Report;
import io.github.hexay.ktrs.maven.ktlint.internal.Sources;
import java.io.File;
import java.util.List;
import java.util.Locale;
import java.util.ResourceBundle;
import java.util.Set;
import org.apache.maven.doxia.siterenderer.Renderer;
import org.apache.maven.plugin.MojoExecutionException;
import org.apache.maven.plugin.descriptor.PluginDescriptor;
import org.apache.maven.plugins.annotations.Component;
import org.apache.maven.plugins.annotations.LifecyclePhase;
import org.apache.maven.plugins.annotations.Mojo;
import org.apache.maven.plugins.annotations.Parameter;
import org.apache.maven.project.MavenProject;
import org.apache.maven.reporting.AbstractMavenReport;

/**
 * A reporting task that performs {@code ktlint} analysis and generates a HTML report on any violations that
 * {@code ktlint} finds.
 */
@Mojo(name = "ktlint", defaultPhase = LifecyclePhase.VERIFY, requiresProject = true, threadSafe = true)
public class KtlintReport extends AbstractMavenReport {
    @Parameter(defaultValue = "${project.basedir}", readonly = true, required = true)
    private File basedir;

    @Parameter(defaultValue = "${project.packaging}", readonly = true, required = true)
    private String packaging;

    @Parameter(defaultValue = "${project.compileSourceRoots}", readonly = true, required = true)
    private List<String> sourceRoots;

    @Parameter(defaultValue = "${project.testCompileSourceRoots}", readonly = true, required = true)
    private List<String> testSourceRoots;

    /**
     * A list of root directories containing Kotlin scripts.
     */
    @Parameter(property = "ktlint.scriptRoots", defaultValue = "${project.basedir.path}")
    private List<String> scriptRoots;

    /**
     * Include the production source roots.
     */
    @Parameter(property = "ktlint.includeSources", defaultValue = "true", required = true)
    private boolean includeSources = true;

    /**
     * Include the test source roots.
     */
    @Parameter(property = "ktlint.includeTestSources", defaultValue = "true", required = true)
    private boolean includeTestSources = true;

    /**
     * Include scripts.
     */
    @Parameter(property = "ktlint.includeScripts", defaultValue = "true", required = true)
    private boolean includeScripts = true;

    /**
     * File encoding of the Kotlin source files.
     */
    @Parameter(property = "encoding", defaultValue = "${project.build.sourceEncoding}")
    private String encoding;

    /**
     * A list of inclusion filters for the source files to be processed under the source roots.
     */
    @Parameter(defaultValue = "**/*.kt")
    private Set<String> sourcesIncludes;

    /**
     * A list of exclusion filters for the source files to be processed under the source roots.
     */
    @Parameter
    private Set<String> sourcesExcludes;

    /**
     * A list of inclusion filters for the source files to be processed under the test source roots.
     */
    @Parameter(defaultValue = "**/*.kt")
    private Set<String> testSourcesIncludes;

    /**
     * A list of exclusion filters for the source files to be processed under the test source roots.
     */
    @Parameter
    private Set<String> testSourcesExcludes;

    /**
     * A list of inclusion filters for scripts.
     */
    @Parameter(defaultValue = "*.kts")
    private Set<String> scriptsIncludes;

    /**
     * A list of exclusion filters for scripts.
     */
    @Parameter
    private Set<String> scriptsExcludes;

    /**
     * Enable Android Kotlin Style Guide compatibility.
     */
    @Parameter(property = "ktlint.android", defaultValue = "false", required = true)
    private boolean android = false;

    /**
     * A set of reporters to output the results to.
     */
    @Parameter
    private Set<ReporterConfig> reporters;

    /**
     * Show error codes.
     */
    @Parameter(property = "ktlint.verbose", defaultValue = "false", required = true)
    private boolean verbose = false;

    /**
     * Enable experimental rules (ktlint-ruleset-experimental).
     */
    @Parameter(property = "ktlint.experimental", defaultValue = "false", required = true)
    private boolean experimental = false;

    /**
     * Skips and code style checks.
     */
    @Parameter(property = "ktlint.skip", defaultValue = "false", required = true)
    private boolean skip = false;

    /**
     * The ktlint version ktrs behaves as: {@code 1.8.0} (gantsign 3.7.1's) or {@code 2.0.0-ALPHA-4}.
     */
    @Parameter(property = "ktrs.ktlintVersion", defaultValue = "1.8.0", required = true)
    private String ktlintVersion;

    /** The plugin's resolved dependencies: the rule set and reporter JARs users add as plugin dependencies. */
    @Parameter(defaultValue = "${plugin}", readonly = true, required = true)
    private PluginDescriptor plugin;

    // AbstractMavenReport's own parameters: the descriptor generator doesn't scan dependency classes, so they are
    // declared here and handed to the base class (see execute).
    @Parameter(defaultValue = "${project.reporting.outputDirectory}", readonly = true, required = true)
    private File reportingOutputDirectory;

    @Parameter(defaultValue = "${project}", readonly = true, required = true)
    private MavenProject mavenProject;

    @Component
    private Renderer siteRendererComponent;

    /** A standalone {@code mvn ktlint:ktlint}: renders the page with the site renderer. */
    @Override
    public void execute() throws MojoExecutionException {
        outputDirectory = reportingOutputDirectory;
        project = mavenProject;
        siteRenderer = siteRendererComponent;
        super.execute();
    }

    private ResourceBundle getBundle(Locale locale) {
        return ResourceBundle.getBundle("ktlint-report", locale, KtlintReport.class.getClassLoader());
    }

    @Override
    public String getName(Locale locale) {
        return KtlintReportGenerator.get(getBundle(locale), "report.ktlint.name");
    }

    @Override
    public String getDescription(Locale locale) {
        return KtlintReportGenerator.get(getBundle(locale), "report.ktlint.description");
    }

    @Override
    public String getOutputName() {
        return "ktlint";
    }

    @Override
    public boolean canGenerateReport() {
        return !skip && sourceRoots.stream().map(File::new).anyMatch(File::isDirectory);
    }

    @Override
    protected void executeReport(Locale locale) {
        CheckResults results;
        try {
            results = new Report(getLog(), basedir, packaging, List.of(
                    new Sources(includeSources, sourceRoots, sourcesIncludes, sourcesExcludes),
                    new Sources(includeTestSources, testSourceRoots, testSourcesIncludes, testSourcesExcludes),
                    new Sources(includeScripts, scriptRoots, scriptsIncludes, scriptsExcludes)),
                    android, reporters == null ? Set.of() : reporters, verbose, experimental, ktlintVersion,
                    plugin.getArtifacts()).invoke();
        } catch (Exception e) {
            throw KtlintReport.<RuntimeException>sneakyThrow(e);
        }
        new KtlintReportGenerator(getSink(), getBundle(locale)).generatorReport(results);
    }

    // Upstream (Kotlin) lets the goal's MojoFailureException escape executeReport unwrapped.
    @SuppressWarnings("unchecked")
    private static <T extends Throwable> T sneakyThrow(Throwable t) throws T {
        throw (T) t;
    }
}
