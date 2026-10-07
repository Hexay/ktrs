package io.github.hexay.ktrs.maven.ktlint.internal;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.io.File;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

class RunErrorsTest {
    @TempDir Path dir;

    private RunErrors read(String content) throws IOException {
        Path report = dir.resolve("report.txt");
        Files.writeString(report, content);
        return RunErrors.read(report.toFile(), dir.toFile());
    }

    @Test
    void events() throws IOException {
        RunErrors errors = read("# ktrs-gradle-events 1\nfile\tsrc/A.kt\n"
                + "error\tsrc/A.kt\t3\t7\tstandard:x\tLINT_CAN_NOT_BE_AUTOCORRECTED\ttrue\tTab\\there\\nnext\n"
                + "file\tsrc/B.kt\n");
        List<KtlintCliError> a = errors.of(dir.resolve("src/A.kt").toFile());
        assertEquals(1, a.size());
        assertEquals(3, a.get(0).line);
        assertEquals(7, a.get(0).col);
        assertEquals("standard:x", a.get(0).ruleId);
        assertEquals("LINT_CAN_NOT_BE_AUTOCORRECTED", a.get(0).status);
        assertTrue(a.get(0).corrected);
        assertEquals("Tab\there\nnext", a.get(0).detail);
        assertTrue(errors.of(dir.resolve("src/B.kt").toFile()).isEmpty());
    }

    @Test
    void jsonReportOfAHandOff() throws IOException {
        RunErrors errors = read("[\n\t{\n\t\t\"file\": \"src/A.kt\",\n\t\t\"errors\": [\n\t\t\t{\n\t\t\t\t\"line\": 2,\n"
                + "\t\t\t\t\"column\": 5,\n\t\t\t\t\"message\": \"Say \\\"hi\\\"\",\n\t\t\t\t\"rule\": \"compose:x\"\n"
                + "\t\t\t}\n\t\t]\n\t}\n]\n");
        KtlintCliError error = errors.of(new File(dir.toFile(), "src" + File.separator + "A.kt")).get(0);
        assertEquals("Say \"hi\"", error.detail);
        assertEquals("compose:x", error.ruleId);
        assertEquals(5, error.col);
        assertNull(error.status);
    }
}
