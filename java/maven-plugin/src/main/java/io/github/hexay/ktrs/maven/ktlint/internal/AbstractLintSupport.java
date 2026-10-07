package io.github.hexay.ktrs.maven.ktlint.internal;

import io.github.hexay.ktrs.KtlintJars;
import java.io.File;
import java.io.IOException;
import java.io.UncheckedIOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.HashSet;
import java.util.List;
import java.util.Set;
import java.util.stream.Stream;
import org.apache.maven.artifact.Artifact;
import org.apache.maven.plugin.MojoExecutionException;
import org.apache.maven.plugin.MojoFailureException;
import org.apache.maven.plugin.logging.Log;
import org.apache.maven.shared.utils.io.DirectoryScanner;

/**
 * gantsign's engine setup, as {@code ktrs ktlint} options: the ktlint version, the plugin {@code <dependencies>}' rule
 * sets ({@code -R}) and the {@code android} / {@code experimental} editorconfig overrides.
 */
abstract class AbstractLintSupport {
    protected final Log log;
    protected final File basedir;
    protected final boolean android;
    protected final boolean enableExperimentalRules;
    protected final String ktlintVersion;
    protected final List<Artifact> pluginArtifacts;

    AbstractLintSupport(Log log, File basedir, boolean android, boolean enableExperimentalRules, String ktlintVersion,
            List<Artifact> pluginArtifacts) {
        this.log = log;
        this.basedir = basedir;
        this.android = android;
        this.enableExperimentalRules = enableExperimentalRules;
        this.ktlintVersion = ktlintVersion;
        this.pluginArtifacts = pluginArtifacts;
    }

    /** What a goal does with one ktrs run: its temporary directory is deleted afterwards. */
    interface KtrsRun<T> {
        T run(KtrsKtlint ktrs, File tempDir) throws MojoExecutionException, MojoFailureException, IOException;
    }

    interface SourceFileAction {
        void accept(File file) throws MojoExecutionException, MojoFailureException;
    }

    protected <T> T withKtrs(KtrsRun<T> run) throws MojoExecutionException, MojoFailureException {
        Path tempDir = null;
        try {
            tempDir = Files.createTempDirectory("ktrs-ktlint-maven");
            return run.run(new KtrsKtlint(basedir, tempDir.toFile(), log), tempDir.toFile());
        } catch (IOException e) {
            throw new MojoExecutionException(e.getMessage(), e);
        } finally {
            if (tempDir != null) deleteRecursively(tempDir);
        }
    }

    /** The options every run takes: version, rule sets and editorconfig overrides. */
    protected List<String> commonOptions(File tempDir) throws MojoFailureException {
        List<String> options = new ArrayList<>();
        options.add(KtlintVersions.cliOption(ktlintVersion));
        if (enableExperimentalRules) {
            log.debug("Add editor config override to allow the experimental rule set");
            options.add("--ktrs-editorconfig-override=ktlint_experimental=enabled");
        }
        if (android) {
            log.debug("Add editor config override to set code style to 'android_studio'");
            options.add("--ktrs-editorconfig-override=ktlint_code_style=android_studio");
        }
        for (File jar : KtlintJars.ruleSetJars(JarServices.userJars(pluginArtifacts), ktlintVersion, tempDir)) {
            options.add("--ruleset=" + jar.getAbsolutePath());
        }
        return options;
    }

    /**
     * gantsign's source walk: each included file once, in source root order. With {@code logging} off it only
     * collects (missing roots are logged by the walk that reports the results, in upstream's order).
     */
    protected void forEachSourceFile(String modulePackaging, List<Sources> sources, String[] defaultIncludes,
            boolean logging, SourceFileAction action) throws MojoExecutionException, MojoFailureException {
        Set<File> checkedFiles = new HashSet<>();
        for (Sources source : sources) {
            if (!source.isIncluded) {
                if (logging) log.debug("Source roots not included: " + source.sourceRoots);
                continue;
            }
            for (File sourceRoot : source.sourceRoots) {
                if (!sourceRoot.exists()) {
                    String msg = "Source root doesn't exist: " + toRelativeString(sourceRoot);
                    if (logging && modulePackaging.equals("pom")) {
                        log.debug(msg);
                    } else if (logging) {
                        log.warn(msg);
                    }
                    continue;
                }
                if (!sourceRoot.isDirectory()) {
                    throw new MojoFailureException("Source root is not a directory: " + toRelativeString(sourceRoot));
                }

                String[] includesArray = source.includes.isEmpty() ? defaultIncludes : source.includes.toArray(new String[0]);
                String[] excludesArray = source.excludes.toArray(new String[0]);

                DirectoryScanner ds = new DirectoryScanner();
                ds.setIncludes(includesArray);
                ds.setExcludes(excludesArray);
                ds.setBasedir(sourceRoot);
                ds.setCaseSensitive(true);
                ds.scan();

                for (String included : ds.getIncludedFiles()) {
                    File file = new File(sourceRoot, included);
                    if (!checkedFiles.add(canonicalFile(file))) continue;
                    action.accept(file);
                }
            }
        }
    }

    /** Kotlin's {@code File.toRelativeString(basedir)}. */
    protected String toRelativeString(File file) {
        return basedir.toPath().relativize(file.toPath()).toString();
    }

    private static File canonicalFile(File file) {
        try {
            return file.getCanonicalFile();
        } catch (IOException e) {
            throw new UncheckedIOException(e);
        }
    }

    private static void deleteRecursively(Path dir) {
        try (Stream<Path> paths = Files.walk(dir)) {
            paths.sorted(Comparator.reverseOrder()).map(Path::toFile).forEach(File::delete);
        } catch (IOException ignored) {
            // Best effort: a leftover temporary directory is harmless.
        }
    }
}
