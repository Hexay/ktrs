package io.github.hexay.ktrs.maven.ktlint.internal;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;

import org.apache.maven.plugin.MojoFailureException;
import org.junit.jupiter.api.Test;

class KtlintVersionsTest {
    @Test
    void supportedVersions() throws MojoFailureException {
        assertEquals("--ktlint-version=1.8", KtlintVersions.cliOption("1.8.0"));
        assertEquals("--ktlint-version=2.0", KtlintVersions.cliOption("2.0.0-ALPHA-4"));
    }

    @Test
    void anyOtherVersionFailsTheBuild() {
        MojoFailureException e = assertThrows(MojoFailureException.class, () -> KtlintVersions.cliOption("1.5.0"));
        assertEquals("ktrs runs ktlint 1.8.0 or 2.0.0-ALPHA-4, not ktlint 1.5.0: set <ktlintVersion> to one of them, "
                + "or use com.github.gantsign.maven:ktlint-maven-plugin for other versions.", e.getMessage());
    }
}
