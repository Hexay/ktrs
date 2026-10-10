package io.github.hexay.ktrs.spotless;

import com.diffplug.spotless.FormatterFunc;
import com.diffplug.spotless.FormatterStep;
import io.github.hexay.ktrs.Ktrs;
import io.github.hexay.ktrs.KtrsOptions;
import java.io.File;
import java.io.Serializable;
import java.util.Objects;

/**
 * A Spotless step (Spotless 7 or later) that formats Kotlin like ktfmt 0.65, through ktrs:
 *
 * <pre>{@code
 * spotless { kotlin { addStep(KtrsStep.create(KtrsOptions.kotlinlang())) } }
 * }</pre>
 */
public final class KtrsStep {
    private KtrsStep() {}

    public static FormatterStep create() {
        return create(KtrsOptions.meta());
    }

    public static FormatterStep create(KtrsOptions options) {
        return FormatterStep.create("ktrs", new State(options), KtrsStep::formatter);
    }

    // Not Ktrs.shared(): its shutdown hook would pin Spotless's classloader in a long-lived daemon.
    private static FormatterFunc formatter(State state) {
        return FormatterFunc.Closeable.of(Ktrs.create(),
                (Ktrs ktrs, String code, File file) -> ktrs.format(code, state.options, file.toPath()));
    }

    /** Spotless's up-to-date key: the options and this jar's version (which pins the binary). */
    private static final class State implements Serializable {
        private static final long serialVersionUID = 1L;

        private final KtrsOptions options;
        private final String version = String.valueOf(KtrsStep.class.getPackage().getImplementationVersion());

        State(KtrsOptions options) {
            this.options = Objects.requireNonNull(options);
        }

        @Override
        public boolean equals(Object o) {
            return o instanceof State && options.equals(((State) o).options) && version.equals(((State) o).version);
        }

        @Override
        public int hashCode() {
            return Objects.hash(options, version);
        }
    }
}
