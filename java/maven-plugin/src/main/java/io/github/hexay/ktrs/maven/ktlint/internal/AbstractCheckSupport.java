package io.github.hexay.ktrs.maven.ktlint.internal;

import io.github.hexay.ktrs.maven.ktlint.MavenLogReporterProvider;
import io.github.hexay.ktrs.maven.ktlint.ReporterConfig;
import java.io.File;
import java.io.IOException;
import java.net.URLEncoder;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.Set;
import java.util.TreeSet;
import java.util.concurrent.atomic.AtomicBoolean;
import org.apache.maven.artifact.Artifact;
import org.apache.maven.plugin.MojoExecutionException;
import org.apache.maven.plugin.MojoFailureException;
import org.apache.maven.plugin.logging.Log;

abstract class AbstractCheckSupport extends AbstractLintSupport {
    // gantsign's classpath: its own `maven` reporter and the ktlint reporters it depends on.
    private static final List<String> BUNDLED_REPORTERS = List.of("baseline", "checkstyle", "json", "maven", "plain");

    private final String modulePackaging;
    private final List<Sources> sources;
    private final Set<ReporterConfig> reporterConfig;
    private final boolean verbose;
    private final boolean reporterColor;
    private final String reporterColorName;

    AbstractCheckSupport(Log log, File basedir, String modulePackaging, List<Sources> sources, boolean android,
            Set<ReporterConfig> reporterConfig, boolean verbose, boolean reporterColor, String reporterColorName,
            boolean enableExperimentalRules, String ktlintVersion, List<Artifact> pluginArtifacts) {
        super(log, basedir, android, enableExperimentalRules, ktlintVersion, pluginArtifacts);
        this.modulePackaging = modulePackaging;
        this.sources = sources;
        this.reporterConfig = reporterConfig;
        this.verbose = verbose;
        this.reporterColor = reporterColor;
        this.reporterColorName = reporterColorName;
    }

    /** The reporters of a run: those in this JVM ({@code maven}), and the {@code --reporter} options of the rest. */
    static final class Reporters {
        final ReporterV2 reporter;
        final List<String> cliOptions;

        Reporters(ReporterV2 reporter, List<String> cliOptions) {
            this.reporter = reporter;
            this.cliOptions = cliOptions;
        }
    }

    private static final class ReporterTemplate {
        final String id;
        final Map<String, String> config;
        final String output;

        ReporterTemplate(String id, Map<String, String> config, String output) {
            this.id = id;
            this.config = config;
            this.output = output;
        }

        @Override
        public boolean equals(Object o) {
            if (!(o instanceof ReporterTemplate)) return false;
            ReporterTemplate other = (ReporterTemplate) o;
            return id.equals(other.id) && config.equals(other.config) && Objects.equals(output, other.output);
        }

        @Override
        public int hashCode() {
            return Objects.hash(id, config, output);
        }
    }

    protected Reporters getReporter() throws MojoFailureException {
        Set<ReporterTemplate> templates = new LinkedHashSet<>();
        for (ReporterConfig reporter : reporterConfig) {
            String reporterId = reporter.getName();
            if (reporterId == null) {
                throw new MojoFailureException("Unable to find reporter without id");
            }
            Map<String, String> config = new LinkedHashMap<>();
            config.put("verbose", String.valueOf(verbose));
            config.put("color", String.valueOf(reporterColor));
            config.put("color_name", reporterColorName);
            if (reporter.getProperties() != null) {
                reporter.getProperties().forEach((key, value) ->
                        config.put(key == null ? "" : key.toString(), value == null ? "" : value.toString()));
            }
            File output = reporter.getOutput();
            templates.add(new ReporterTemplate(reporterId, config, output == null ? null : output.toString()));
        }

        Set<String> available = new TreeSet<>(BUNDLED_REPORTERS);
        available.addAll(JarServices.ktlintReporterIds(pluginArtifacts));
        List<File> reporterJars = JarServices.reporterJars(JarServices.userJars(pluginArtifacts), ktlintVersion);
        for (String id : available) {
            log.debug("Discovered reporter '" + id + "'");
        }

        List<ReporterV2> reporters = new ArrayList<>();
        List<String> cliOptions = new ArrayList<>();
        int customReporters = 0;
        for (ReporterTemplate template : templates) {
            boolean custom = !available.contains(template.id);
            if (custom && reporterJars.isEmpty()) {
                throw new MojoFailureException(
                        "Error: reporter '" + template.id + "' wasn't found (available: " + String.join(",", available) + ")");
            }
            if (log.isDebugEnabled()) {
                log.debug("Initializing '" + template.id + "' reporter with " + template.config
                        + (template.output == null ? "" : ", output=" + template.output));
            }
            if (template.id.equals("maven")) {
                createOutput(template.output);
                reporters.add(new MavenLogReporterProvider().get(log, template.config));
            } else {
                File artifact = custom ? reporterJars.get(customReporters++ % reporterJars.size()) : null;
                cliOptions.add(cliOption(template, artifact));
            }
        }
        return new Reporters(new AggregatedReporter(reporters), cliOptions);
    }

    /**
     * {@code --reporter=<id>?<properties>[,artifact=<jar>][,output=<file>]}; the CLI sets {@code color} and
     * {@code color_name} itself, from {@code --color} / {@code --color-name}.
     */
    private static String cliOption(ReporterTemplate template, File artifact) {
        StringBuilder query = new StringBuilder();
        template.config.forEach((key, value) -> {
            if (key.equals("color") || key.equals("color_name")) return;
            query.append(query.length() == 0 ? "?" : "&").append(key).append('=')
                    .append(URLEncoder.encode(value, StandardCharsets.UTF_8));
        });
        StringBuilder option = new StringBuilder("--reporter=").append(template.id).append(query);
        if (artifact != null) option.append(",artifact=").append(artifact.getAbsolutePath());
        if (template.output != null) option.append(",output=").append(new File(template.output).getAbsolutePath());
        return option.toString();
    }

    /** gantsign opens a stream on every reporter's {@code output}, the {@code maven} reporter's included. */
    private static void createOutput(String output) throws MojoFailureException {
        if (output == null) return;
        try {
            File file = new File(output);
            Files.createDirectories(file.getAbsoluteFile().getParentFile().toPath());
            Files.write(file.toPath(), new byte[0]);
        } catch (IOException e) {
            throw new MojoFailureException(e.getMessage(), e);
        }
    }

    protected boolean hasErrors(Reporters reporters) throws MojoExecutionException, MojoFailureException {
        return withKtrs((ktrs, tempDir) -> {
            List<String> options = commonOptions(tempDir);
            options.add("--relative");
            options.add("--ktrs-relative-to=" + basedir.getAbsolutePath());
            if (reporterColor) options.add("--color");
            options.add("--color-name=" + reporterColorName);
            File events = new File(tempDir, "events.txt");
            options.add("--ktrs-gradle-events=" + events.getAbsolutePath());
            // Without a --reporter the CLI prints `plain` to stdout.
            if (reporters.cliOptions.isEmpty()) options.add("--reporter=plain,output=" + new File(tempDir, "plain.txt"));
            options.addAll(reporters.cliOptions);

            List<File> files = new ArrayList<>();
            forEachSourceFile(modulePackaging, sources, new String[] {"**/*.kt", "**/*.kts"}, false, files::add);
            ktrs.run(options, files);
            RunErrors runErrors = RunErrors.read(events, basedir);

            AtomicBoolean hasErrors = new AtomicBoolean();
            ReporterV2 reporter = reporters.reporter;
            reporter.beforeAll();
            forEachSourceFile(modulePackaging, sources, new String[] {"**/*.kt", "**/*.kts"}, true, file -> {
                String baseRelativePath = toRelativeString(file);
                log.debug("checking: " + baseRelativePath);
                List<KtlintCliError> ktlintCliErrors = runErrors.of(file);
                report(baseRelativePath, ktlintCliErrors, reporter);
                for (KtlintCliError it : ktlintCliErrors) {
                    log.debug("Style error > " + baseRelativePath + ":" + it.line + ":" + it.col + ": " + it.detail);
                }
                if (!ktlintCliErrors.isEmpty()) hasErrors.set(true);
            });
            reporter.afterAll();
            return hasErrors.get();
        });
    }

    private static void report(String relativeRoute, List<KtlintCliError> ktlintCliErrors, ReporterV2 reporter) {
        reporter.before(relativeRoute);
        ktlintCliErrors.forEach(it -> reporter.onLintError(relativeRoute, it));
        reporter.after(relativeRoute);
    }
}
