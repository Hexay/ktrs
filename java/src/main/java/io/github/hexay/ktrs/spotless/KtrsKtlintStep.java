package io.github.hexay.ktrs.spotless;

import com.diffplug.spotless.FileSignature;
import com.diffplug.spotless.FormatterFunc;
import com.diffplug.spotless.FormatterStep;
import com.diffplug.spotless.Lint;
import io.github.hexay.ktrs.KtlintOptions;
import io.github.hexay.ktrs.KtlintResult;
import io.github.hexay.ktrs.Ktrs;
import java.io.File;
import java.io.Serializable;
import java.util.ArrayList;
import java.util.List;
import java.util.Objects;

/**
 * A Spotless step (Spotless 7 or later) that does what Spotless's {@code ktlint()} step does, through ktrs:
 *
 * <pre>{@code
 * spotless { kotlin { addStep(KtrsKtlintStep.create(KtlintOptions.defaults()
 *     .withEditorConfigOverride(mapOf("indent_size" to 2)))) } }
 * }</pre>
 *
 * Like Spotless's step it formats, and when a violation can't be autocorrected it reports the first one as a
 * lint and leaves the file to the next step unchanged (research/28-spotless-ktlint-step.md).
 */
public final class KtrsKtlintStep {
    /** Spotless's {@code ktlint()} step's name, so lints read the same. */
    static final String NAME = "ktlint";

    private KtrsKtlintStep() {}

    public static FormatterStep create() {
        return create(KtlintOptions.defaults());
    }

    /** @param version {@value KtlintOptions#DEFAULT_VERSION} or {@value KtlintOptions#VERSION_2_0} */
    public static FormatterStep create(String version) {
        return create(KtlintOptions.of(version));
    }

    public static FormatterStep create(KtlintOptions options) {
        return FormatterStep.create(NAME, new Roundtrip(options), Roundtrip::toEquality, KtrsKtlintStep::formatter);
    }

    // Not Ktrs.shared(): its shutdown hook would pin Spotless's classloader in a long-lived daemon.
    private static FormatterFunc formatter(Equality state) {
        return FormatterFunc.Closeable.of(Ktrs.create(), (Ktrs ktrs, String code, File file) -> {
            KtlintResult result = ktrs.ktlint(code, state.options, file.toPath());
            if (!result.violations().isEmpty()) {
                KtlintResult.Violation first = result.violations().get(0);
                throw Lint.atLine(first.line(), first.ruleId(), first.detail()).shortcut();
            }
            return result.code();
        });
    }

    private static List<File> inputFiles(KtlintOptions options) {
        List<File> files = new ArrayList<>(options.customRuleSets());
        if (options.editorConfigPath() != null) {
            files.add(options.editorConfigPath());
        }
        return files;
    }

    /** What Gradle's configuration cache keeps: the options, with the input files signed lazily. */
    private static final class Roundtrip implements Serializable {
        private static final long serialVersionUID = 1L;

        private final KtlintOptions options;
        private final FileSignature.Promised files;

        Roundtrip(KtlintOptions options) {
            this.options = Objects.requireNonNull(options);
            this.files = FileSignature.promise(inputFiles(options));
        }

        Equality toEquality() {
            return new Equality(options, files.get());
        }
    }

    /** Spotless's up-to-date key: the options, the input files' contents and this jar's version (which pins the binary). */
    private static final class Equality implements Serializable {
        private static final long serialVersionUID = 1L;

        private final KtlintOptions options;
        private final FileSignature files;
        private final String version = String.valueOf(KtrsKtlintStep.class.getPackage().getImplementationVersion());

        Equality(KtlintOptions options, FileSignature files) {
            this.options = options;
            this.files = files;
        }
    }
}
