package io.github.hexay.ktrs.spotless.maven;

import com.diffplug.spotless.FormatterStep;
import com.diffplug.spotless.maven.FormatterStepConfig;
import com.diffplug.spotless.maven.kotlin.Ktlint;
import io.github.hexay.ktrs.KtlintOptions;
import io.github.hexay.ktrs.spotless.KtrsKtlintStep;
import java.io.File;
import java.util.List;
import java.util.Map;

/**
 * spotless-maven-plugin's {@code <ktlint>}, linting and formatting through ktrs instead of the ktlint jar:
 *
 * <pre>{@code
 * <ktlint implementation="io.github.hexay.ktrs.spotless.maven.KtrsKtlint"/>
 * }</pre>
 *
 * Takes the same options. {@code version} is {@value KtlintOptions#DEFAULT_VERSION} (the default, as in Spotless) or
 * {@value KtlintOptions#VERSION_2_0}; {@code customRuleSets} runs the compose-rules release ktrs ports and fails on any
 * other rule set. Field shadowing: see {@link KtrsKtfmt}.
 */
public class KtrsKtlint extends Ktlint {
    private String version;
    private String editorConfigPath;
    private Map<String, Object> editorConfigOverride;
    private List<String> customRuleSets;

    @Override
    public FormatterStep newFormatterStep(FormatterStepConfig config) {
        KtlintOptions options = KtlintOptions.of(version == null ? KtlintOptions.DEFAULT_VERSION : version);
        String path = editorConfigPath;
        // Spotless's default: `.editorconfig` in the working directory, not the project's basedir.
        if (path == null && new File(".editorconfig").exists()) path = ".editorconfig";
        if (path != null) options = options.withEditorConfigPath(config.getFileLocator().locateFile(path));
        if (editorConfigOverride != null) options = options.withEditorConfigOverride(editorConfigOverride);
        if (customRuleSets != null && !customRuleSets.isEmpty()) {
            File dir = new File(config.getFileLocator().getBuildDir(), "ktrs");
            options = options.withCustomRuleSets(RuleSetJars.of(customRuleSets,
                    config.getProvisioner().provisionWithTransitives(true, customRuleSets), dir));
        }
        return KtrsKtlintStep.create(options);
    }
}
