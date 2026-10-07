package io.github.hexay.ktrs.maven.ktlint.internal;

import java.io.File;
import java.util.Collection;
import java.util.LinkedHashSet;
import java.util.Set;
import java.util.stream.Collectors;

public final class Sources {
    final boolean isIncluded;
    final Set<File> sourceRoots;
    final Set<String> includes;
    final Set<String> excludes;

    public Sources(boolean isIncluded, Collection<String> sourceRoots, Collection<String> includes,
            Collection<String> excludes) {
        this.isIncluded = isIncluded;
        this.sourceRoots = sourceRoots == null
                ? Set.of()
                : sourceRoots.stream().map(File::new).collect(Collectors.toCollection(LinkedHashSet::new));
        this.includes = includes == null ? Set.of() : new LinkedHashSet<>(includes);
        this.excludes = excludes == null ? Set.of() : new LinkedHashSet<>(excludes);
    }
}
