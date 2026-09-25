import com.facebook.ktfmt.format.Formatter;
import com.facebook.ktfmt.format.FormattingOptions;
import com.facebook.ktfmt.format.TrailingCommaManagementStrategy;
import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;
import java.util.Properties;
import java.util.stream.Stream;

/**
 * Batch oracle over golden cases: for every {@code <name>.input.kt} + {@code <name>.options} under
 * the given dir, calls {@code Formatter.format(options, code)} of the real ktfmt and writes
 * {@code <name>.expected.kt}, or {@code <name>.error} when ktfmt rejects the input. When a
 * {@code <name>.upstream.kt} (the upstream test's own expectation) exists, disagreements are listed.
 */
public final class KtfmtOracle {
  public static void main(String[] args) throws IOException {
    if (args.length != 1) {
      System.err.println("usage: KtfmtOracle <cases-dir>");
      System.exit(2);
    }
    List<Path> inputs;
    try (Stream<Path> walk = Files.walk(Path.of(args[0]))) {
      inputs = walk.filter(p -> p.toString().endsWith(".input.kt")).sorted().toList();
    }
    int ok = 0, rejected = 0, disagree = 0;
    for (Path input : inputs) {
      String base = input.toString().substring(0, input.toString().length() - ".input.kt".length());
      Path expected = Path.of(base + ".expected.kt"), error = Path.of(base + ".error");
      Files.deleteIfExists(expected);
      Files.deleteIfExists(error);
      String code = Files.readString(input, StandardCharsets.UTF_8);
      FormattingOptions options = readOptions(Path.of(base + ".options"));
      String output;
      try {
        output = Formatter.format(options, code);
      } catch (Exception | StackOverflowError e) {
        rejected++;
        Files.writeString(error, e.getClass().getSimpleName() + ": " + e.getMessage() + "\n");
        output = null;
      }
      if (output != null) {
        ok++;
        Files.writeString(expected, output, StandardCharsets.UTF_8);
      }
      Path upstream = Path.of(base + ".upstream.kt");
      if (Files.exists(upstream)
          && !Files.readString(upstream, StandardCharsets.UTF_8).equals(output)) {
        disagree++;
        System.out.println("DISAGREE " + base);
      }
    }
    System.out.printf(
        "oracle: %d cases, %d formatted, %d rejected, %d disagree with upstream%n",
        inputs.size(), ok, rejected, disagree);
  }

  private static FormattingOptions readOptions(Path path) throws IOException {
    Properties p = new Properties();
    try (var reader = Files.newBufferedReader(path, StandardCharsets.UTF_8)) {
      p.load(reader);
    }
    return new FormattingOptions.Builder()
        .maxWidth(Integer.parseInt(p.getProperty("maxWidth")))
        .blockIndent(Integer.parseInt(p.getProperty("blockIndent")))
        .continuationIndent(Integer.parseInt(p.getProperty("continuationIndent")))
        .trailingCommaManagementStrategy(
            TrailingCommaManagementStrategy.valueOf(p.getProperty("trailingCommaManagementStrategy")))
        .removeUnusedImports(Boolean.parseBoolean(p.getProperty("removeUnusedImports")))
        .preserveLambdaBreaks(Boolean.parseBoolean(p.getProperty("preserveLambdaBreaks")))
        .debuggingPrintOpsAfterFormatting(
            Boolean.parseBoolean(p.getProperty("debuggingPrintOpsAfterFormatting")))
        .build();
  }
}
