package io.github.hexay.ktrs.maven.ktlint.internal;

import static org.junit.jupiter.api.Assertions.assertEquals;

import org.junit.jupiter.api.Test;

class FormatTest {
    @Test
    void parseExceptionMessageFromTheCliDetail() {
        assertEquals("3:12 Expecting ')'", Format.exceptionMessage("Not a valid Kotlin file (3:12 expecting ')')"));
    }

    @Test
    void otherDetailsAsIs() {
        assertEquals("Internal Error (rule 'x')", Format.exceptionMessage("Internal Error (rule 'x')"));
    }
}
