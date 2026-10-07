package io.github.hexay.ktrs.spotless.maven;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import com.diffplug.spotless.kotlin.KtLintStep;
import com.diffplug.spotless.kotlin.KtfmtStep;
import io.github.hexay.ktrs.KtlintOptions;
import java.lang.reflect.Field;
import java.lang.reflect.Modifier;
import java.util.Arrays;
import java.util.Set;
import java.util.TreeSet;
import java.util.stream.Collectors;
import org.junit.jupiter.api.Test;

/**
 * The swap only sees options through its shadowing fields (see {@link KtrsKtfmt}): when the pinned
 * spotless-maven-plugin adds, renames or retypes one, this fails instead of the option silently going unset.
 */
class KtrsSpotlessMavenFieldsTest {
    @Test
    void ktfmtShadowsEveryOption() {
        assertEquals(options(KtrsKtfmt.class.getSuperclass()), options(KtrsKtfmt.class));
    }

    @Test
    void ktlintShadowsEveryOption() {
        assertEquals(options(KtrsKtlint.class.getSuperclass()), options(KtrsKtlint.class));
    }

    /** Without {@code <version>}, stock Spotless runs its default releases: the ones ktrs matches. */
    @Test
    void defaultVersionsAreSpotlessDefaults() {
        assertEquals(KtfmtStep.defaultVersion(), KtrsKtfmt.KTFMT_VERSION);
        assertEquals(KtLintStep.defaultVersion(), KtlintOptions.DEFAULT_VERSION);
    }

    @Test
    void ktfmtRejectsAnotherVersion() {
        KtrsKtfmt ktfmt = new KtrsKtfmt();
        set(ktfmt, "version", "0.61");
        String message = assertThrows(IllegalArgumentException.class, () -> ktfmt.newFormatterStep(null)).getMessage();
        assertTrue(message.startsWith("ktrs formats like ktfmt 0.64, not 0.61"), message);
    }

    @Test
    void ktfmtRejectsAStyleKtfmt064Lacks() {
        KtrsKtfmt ktfmt = new KtrsKtfmt();
        set(ktfmt, "style", "DROPBOX");
        String message = assertThrows(IllegalArgumentException.class, () -> ktfmt.newFormatterStep(null)).getMessage();
        assertEquals("ktfmt 0.64 has no style DROPBOX: use one of [META, GOOGLE, KOTLINLANG]", message);
    }

    @Test
    void ktlintRejectsAnotherVersion() {
        KtrsKtlint ktlint = new KtrsKtlint();
        set(ktlint, "version", "1.7.1");
        String message = assertThrows(IllegalArgumentException.class, () -> ktlint.newFormatterStep(null)).getMessage();
        assertEquals("ktrs matches ktlint 1.8.0 and 2.0.0-ALPHA-4, not 1.7.1", message);
    }

    private static Set<String> options(Class<?> type) {
        return Arrays.stream(type.getDeclaredFields())
                .filter(field -> !Modifier.isStatic(field.getModifiers()) && !field.isSynthetic())
                .map(field -> field.getName() + ": " + field.getGenericType().getTypeName())
                .collect(Collectors.toCollection(TreeSet::new));
    }

    private static void set(Object target, String name, Object value) {
        try {
            Field field = target.getClass().getDeclaredField(name);
            field.setAccessible(true);
            field.set(target, value);
        } catch (ReflectiveOperationException e) {
            throw new AssertionError(e);
        }
    }
}
