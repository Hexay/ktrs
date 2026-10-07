package io.github.hexay.ktrs.spotless.maven;

import com.diffplug.spotless.FormatterStep;
import com.diffplug.spotless.kotlin.KtfmtStep.TrailingCommaManagementStrategy;
import com.diffplug.spotless.maven.FormatterStepConfig;
import com.diffplug.spotless.maven.kotlin.Ktfmt;
import io.github.hexay.ktrs.KtrsOptions;
import io.github.hexay.ktrs.spotless.KtrsStep;
import java.util.Arrays;

/**
 * spotless-maven-plugin's {@code <ktfmt>}, formatting through ktrs instead of the ktfmt jar:
 *
 * <pre>{@code
 * <ktfmt implementation="io.github.hexay.ktrs.spotless.maven.KtrsKtfmt"><style>KOTLINLANG</style></ktfmt>
 * }</pre>
 *
 * Takes the same options. The fields shadow {@link Ktfmt}'s private ones, which Maven then leaves unset: it injects
 * into the most-derived field of a name (KtrsSpotlessMavenFieldsTest keeps the two sets equal).
 */
public class KtrsKtfmt extends Ktfmt {
    /** The only ktfmt release ktrs formats like. */
    static final String KTFMT_VERSION = "0.64";

    private String version;
    private String style;
    private Integer maxWidth;
    private Integer blockIndent;
    private Integer continuationIndent;
    private Boolean removeUnusedImports;
    private TrailingCommaManagementStrategy trailingCommaManagementStrategy;

    @Override
    public FormatterStep newFormatterStep(FormatterStepConfig config) {
        if (version != null && !version.equals(KTFMT_VERSION)) {
            throw new IllegalArgumentException("ktrs formats like ktfmt " + KTFMT_VERSION + ", not " + version
                    + ": set <version>" + KTFMT_VERSION + "</version> or drop it, or remove implementation=\""
                    + KtrsKtfmt.class.getName() + "\" to run that ktfmt release");
        }
        KtrsOptions options = KtrsOptions.of(style == null ? KtrsOptions.Style.META : style(style));
        if (maxWidth != null) options = options.withMaxWidth(maxWidth);
        if (blockIndent != null) options = options.withBlockIndent(blockIndent);
        if (continuationIndent != null) options = options.withContinuationIndent(continuationIndent);
        if (removeUnusedImports != null) options = options.withRemoveUnusedImports(removeUnusedImports);
        if (trailingCommaManagementStrategy != null) {
            options = options.withTrailingCommas(KtrsOptions.TrailingCommas.valueOf(trailingCommaManagementStrategy.name()));
        }
        return KtrsStep.create(options);
    }

    private static KtrsOptions.Style style(String name) {
        try {
            return KtrsOptions.Style.valueOf(name);
        } catch (IllegalArgumentException e) {
            throw new IllegalArgumentException("ktfmt " + KTFMT_VERSION + " has no style " + name + ": use one of "
                    + Arrays.toString(KtrsOptions.Style.values()), e);
        }
    }
}
