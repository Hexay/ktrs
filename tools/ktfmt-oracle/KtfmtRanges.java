import java.io.ByteArrayInputStream;
import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.io.PrintStream;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import org.jetbrains.kotlinx.ktfmt.cli.Main;

/**
 * Runs the real ktfmt CLI in one JVM over range-diff.sh's plan: {@code KtfmtRanges <plan.tsv>
 * <work-dir>}. A plan line is {@code <id>\t<source>\t<file name>\t<flags, space separated>}; the
 * case's copy is {@code <work-dir>/<id>/<file name>}, formatted in place, and the exit code goes
 * to {@code <work-dir>/<id>/code}.
 */
public final class KtfmtRanges {
  public static void main(String[] args) throws IOException {
    Path work = Path.of(args[1]);
    int cases = 0;
    for (String line : Files.readAllLines(Path.of(args[0]), StandardCharsets.UTF_8)) {
      String[] fields = line.split("\t");
      Path dir = work.resolve(fields[0]);
      List<String> arguments = new ArrayList<>(List.of(fields[3].split(" ")));
      arguments.add(dir.resolve(fields[2]).toString());
      PrintStream sink = new PrintStream(new ByteArrayOutputStream());
      String code;
      try {
        code =
            String.valueOf(
                new Main(new ByteArrayInputStream(new byte[0]), sink, sink, arguments.toArray(new String[0]))
                    .run());
      } catch (Throwable t) {
        code = t.getClass().getName();
      }
      Files.writeString(dir.resolve("code"), code + "\n");
      cases++;
    }
    System.out.println("jar: " + cases + " cases");
  }
}
