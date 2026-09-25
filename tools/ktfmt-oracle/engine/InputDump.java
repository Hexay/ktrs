import com.facebook.ktfmt.format.KotlinInput;
import com.facebook.ktfmt.format.Parser;
import com.google.common.collect.Range;
import com.google.googlejavaformat.Input;
import java.io.*;
import java.nio.charset.StandardCharsets;
import java.nio.file.*;
import java.util.*;
import java.util.stream.*;

public class InputDump {
  static String esc(String s) {
    StringBuilder b = new StringBuilder("\"");
    for (int i = 0; i < s.length(); i++) {
      char c = s.charAt(i);
      switch (c) {
        case '\\': b.append("\\\\"); break;
        case '"': b.append("\\\""); break;
        case '\n': b.append("\\n"); break;
        case '\r': b.append("\\r"); break;
        case '\t': b.append("\\t"); break;
        default:
          if (c < 0x20 || c == 0x7f) b.append(String.format("\\u%04x", (int) c));
          else b.append(c);
      }
    }
    return b.append('"').toString();
  }

  static String tok(Input.Tok t) {
    String s = t.getIndex() + "@" + t.getPosition() + ":" + esc(t.getText());
    if (!t.getText().equals(t.getOriginalText())) s += "/" + esc(t.getOriginalText());
    return s;
  }

  public static String dump(String code) {
    StringBuilder out = new StringBuilder();
    try {
      KotlinInput input = new KotlinInput(code, Parser.INSTANCE.parse(code));
      out.append("kN ").append(input.getkN()).append('\n');
      for (Input.Token t : input.getTokens()) {
        out.append("B");
        for (Input.Tok x : t.getToksBefore()) out.append(' ').append(tok(x));
        out.append("\nT ").append(tok(t.getTok())).append("\nA");
        for (Input.Tok x : t.getToksAfter()) out.append(' ').append(tok(x));
        out.append('\n');
      }
      out.append("lines ").append(input.getLineCount()).append('\n');
      for (int i = 0; i <= input.getLineCount(); i++) {
        Range<Integer> r = input.getRanges(i);
        out.append("R ").append(r.lowerEndpoint()).append(' ').append(r.upperEndpoint()).append('\n');
      }
    } catch (Throwable e) {
      out.setLength(0);
      out.append("ERROR ").append(e.getClass().getSimpleName()).append(' ').append(e.getMessage()).append('\n');
    }
    return out.toString();
  }

  public static void main(String[] args) throws Exception {
    Path root = Paths.get(args[0]);
    Path outRoot = Paths.get(args[1]);
    List<Path> files;
    try (Stream<Path> s = Files.walk(root)) {
      files = s.filter(p -> p.toString().endsWith(".kt") || p.toString().endsWith(".kts")).sorted().collect(Collectors.toList());
    }
    int n = 0;
    for (Path p : files) {
      String code = new String(Files.readAllBytes(p), StandardCharsets.UTF_8).replace("\r\n", "\n").replace('\r', '\n');
      Path out = outRoot.resolve(root.relativize(p).toString() + ".tokens");
      Files.createDirectories(out.getParent());
      Files.write(out, dump(code).getBytes(StandardCharsets.UTF_8));
      n++;
    }
    System.out.println("dumped " + n);
  }
}
