import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Properties;
import java.util.regex.Matcher;
import java.util.regex.Pattern;
import java.util.stream.Stream;
import org.jetbrains.kotlinx.ktfmt.format.FileType;
import org.jetbrains.kotlinx.ktfmt.format.Formatter;
import org.jetbrains.kotlinx.ktfmt.format.FormattingOptions;
import org.jetbrains.kotlinx.ktfmt.format.KotlinCode;
import org.jetbrains.kotlinx.ktfmt.format.TrailingCommaManagementStrategy;

/**
 * Batch oracle over golden cases, on the real ktfmt jar.
 *
 * <p>{@code KtfmtOracle <cases-dir>}: for every {@code <name>.input.kt} + {@code <name>.options}
 * under the dir, calls {@code Formatter.format(options, KotlinCode(code, fileType))} and writes
 * {@code <name>.expected.kt}, or {@code <name>.error} when ktfmt rejects the input. When a {@code
 * <name>.upstream.kt} (the upstream test's own expectation) exists, disagreements are listed.
 *
 * <p>{@code KtfmtOracle import <upstream cases/<group>> <style> <out-dir>}: first turns upstream's
 * file-based cases ({@code <name>.input}, optional {@code <name>.output}, directive header: see its
 * {@code CaseConfig.kt}) into that layout, with the group's style as the base options.
 */
public final class KtfmtOracle {
  public static void main(String[] args) throws IOException {
    if (args.length == 4 && args[0].equals("import")) {
      importCases(Path.of(args[1]), args[2], Path.of(args[3]));
      return;
    }
    if (args.length != 1) {
      System.err.println("usage: KtfmtOracle <cases-dir> | import <upstream-group-dir> <style> <out-dir>");
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
      Properties p = new Properties();
      try (var reader = Files.newBufferedReader(Path.of(base + ".options"), StandardCharsets.UTF_8)) {
        p.load(reader);
      }
      String output;
      try {
        output =
            Formatter.format(
                readOptions(p), KotlinCode.Companion.from(code, FileType.valueOf(p.getProperty("fileType"))));
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

  private static FormattingOptions readOptions(Properties p) {
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

  private static void importCases(Path group, String style, Path out) throws IOException {
    FormattingOptions base =
        switch (style) {
          case "meta" -> Formatter.META_FORMAT;
          case "google" -> Formatter.GOOGLE_FORMAT;
          case "kotlinlang" -> Formatter.KOTLINLANG_FORMAT;
          default -> throw new IllegalArgumentException("unknown style " + style);
        };
    List<Path> inputs;
    try (Stream<Path> walk = Files.walk(group)) {
      inputs = walk.filter(p -> p.toString().endsWith(".input")).sorted().toList();
    }
    for (Path input : inputs) {
      String relative = group.relativize(input).toString().replace('\\', '/');
      String name = relative.substring(0, relative.length() - ".input".length());
      String code = Files.readString(input, StandardCharsets.UTF_8);
      Path target = out.resolve(name + ".input.kt");
      Files.createDirectories(target.getParent());
      Files.writeString(target, code, StandardCharsets.UTF_8);
      Files.writeString(out.resolve(name + ".options"), options(base, directives(code, input)));
      Path output = input.resolveSibling(input.getFileName().toString().replaceFirst("\\.input$", ".output"));
      // No .output: formatting the input is expected to be a no-op.
      String upstream = Files.isRegularFile(output) ? Files.readString(output, StandardCharsets.UTF_8) : code;
      Files.writeString(out.resolve(name + ".upstream.kt"), upstream, StandardCharsets.UTF_8);
    }
    System.out.printf("imported %d cases from %s%n", inputs.size(), group);
  }

  private static final Pattern DIRECTIVE = Pattern.compile("^// ([A-Z][A-Z0-9_]+)(?: +(.*))?$");

  /** {@code CaseConfig.parse}: the directive comments heading the input, after any shebang line. */
  private static Map<String, String> directives(String code, Path origin) {
    Map<String, String> directives = new LinkedHashMap<>();
    List<String> lines = code.lines().toList();
    int start = 0;
    if (!lines.isEmpty() && lines.get(0).startsWith("#!")) {
      directives.put("FILE_TYPE", "SCRIPT");
      start = 1;
    }
    for (int i = start; i < lines.size() && lines.get(i).startsWith("//"); i++) {
      Matcher m = DIRECTIVE.matcher(lines.get(i));
      if (m.matches()) {
        directives.put(m.group(1), m.group(2));
      }
    }
    return directives;
  }

  private static String options(FormattingOptions base, Map<String, String> directives) {
    Map<String, String> o = new LinkedHashMap<>();
    o.put("fileType", "REGULAR");
    o.put("maxWidth", String.valueOf(base.getMaxWidth()));
    o.put("blockIndent", String.valueOf(base.getBlockIndent()));
    o.put("continuationIndent", String.valueOf(base.getContinuationIndent()));
    o.put("trailingCommaManagementStrategy", base.getTrailingCommaManagementStrategy().name());
    o.put("removeUnusedImports", String.valueOf(base.getRemoveUnusedImports()));
    o.put("preserveLambdaBreaks", String.valueOf(base.getPreserveLambdaBreaks()));
    o.put("debuggingPrintOpsAfterFormatting", "false");
    directives.forEach(
        (name, value) -> {
          switch (name) {
            case "FILE_TYPE" -> o.put("fileType", value);
            case "MAX_WIDTH" -> o.put("maxWidth", value);
            case "BLOCK_INDENT" -> o.put("blockIndent", value);
            case "CONTINUATION_INDENT" -> o.put("continuationIndent", value);
            case "TRAILING_COMMA_STRATEGY" -> o.put("trailingCommaManagementStrategy", value);
            case "REMOVE_UNUSED_IMPORTS" -> o.put("removeUnusedImports", value);
            case "PRESERVE_LAMBDA_BREAKS" -> o.put("preserveLambdaBreaks", value);
            // The op dump goes to stdout, not into the result.
            case "PRINT_OPTS_AFTER_FORMATTING", "CHECK_IDEMPOTENCY" -> {}
            default -> throw new IllegalStateException(name + ": unknown directive");
          }
        });
    StringBuilder text = new StringBuilder();
    o.forEach((key, value) -> text.append(key).append('=').append(value).append('\n'));
    return text.toString();
  }
}
