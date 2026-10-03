package io.github.hexay.ktrs;

import static java.nio.charset.StandardCharsets.UTF_8;

import java.io.IOException;
import java.io.UncheckedIOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.stream.Collectors;
import java.util.stream.Stream;

/** Inputs for the ktlint parity tests: built-in samples, or the files of -PktlintParityCorpus=dir (LF line ends). */
public final class KtlintSamples {
    private static final Map<String, String> SAMPLES = new LinkedHashMap<>();

    static {
        SAMPLES.put("Indent.kt", "fun  f() {\n  val x = 1\n}\n");
        SAMPLES.put("Wildcard.kt", "import a.*\nimport b.c\n\nfun f() = c()\n");
        SAMPLES.put("Empty.kt", "");
        SAMPLES.put("Broken.kt", "fun f( {\n");
        SAMPLES.put("Params.kt", "fun f(\n    a: Int,\n    b: Int\n) = a + b\n");
        SAMPLES.put("Blank.kt", "class A {\n\n\n    fun g() = listOf(1,2,3).map { it*2 }\n}\n");
        SAMPLES.put("Compose.kt", "@Composable\nfun MyComposable() {\n    Text(\"x\")\n}\n");
        SAMPLES.put("Long.kt", "val s = \"" + "x".repeat(150) + "\"\n");
        SAMPLES.put("Script.kts", "plugins {\n  id(\"x\")\n}\n");
        SAMPLES.put("Doc.kt", "/**\n * Doc\n */\nfun f() {\n    // comment\n    if (true) { println() }\n}\n");
    }

    private KtlintSamples() {}

    /** Writes the inputs under {@code dir} (a root {@code .editorconfig} for the samples); their unformatted text. */
    public static Map<Path, String> write(Path dir) {
        try {
            String corpus = System.getProperty("ktrs.ktlintParityCorpus");
            return corpus == null ? writeSamples(dir) : copyCorpus(Paths.get(corpus), dir);
        } catch (IOException e) {
            throw new UncheckedIOException(e);
        }
    }

    private static Map<Path, String> writeSamples(Path dir) throws IOException {
        Files.writeString(dir.resolve(".editorconfig"), "root = true\n");
        Map<Path, String> files = new LinkedHashMap<>();
        for (Map.Entry<String, String> sample : SAMPLES.entrySet()) {
            Path file = dir.resolve(sample.getKey());
            Files.writeString(file, sample.getValue());
            files.put(file, sample.getValue());
        }
        return files;
    }

    private static Map<Path, String> copyCorpus(Path corpus, Path dir) throws IOException {
        int max = Integer.getInteger("ktrs.ktlintParityMaxFiles", 300);
        List<Path> sources;
        try (Stream<Path> walk = Files.walk(corpus)) {
            sources = walk.filter(Files::isRegularFile).sorted().collect(Collectors.toList());
        }
        Map<Path, String> files = new LinkedHashMap<>();
        for (Path source : sources) {
            String name = source.getFileName().toString();
            boolean kotlin = name.endsWith(".kt") || name.endsWith(".kts");
            if (!kotlin && !name.equals(".editorconfig") || kotlin && files.size() >= max) {
                continue;
            }
            Path target = dir.resolve(corpus.relativize(source).toString());
            Files.createDirectories(target.getParent());
            String text = new String(Files.readAllBytes(source), UTF_8).replace("\r\n", "\n");
            Files.writeString(target, text);
            if (kotlin) {
                files.put(target, text);
            }
        }
        return files;
    }
}
