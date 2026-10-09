import com.google.common.collect.ImmutableList;
import com.google.common.collect.Range;
import com.google.googlejavaformat.*;
import com.google.googlejavaformat.Output.BreakTag;
import com.google.googlejavaformat.java.JavaOutput;
import java.nio.charset.StandardCharsets;
import java.nio.file.*;
import java.util.*;
import org.jetbrains.kotlinx.ktfmt.format.FenceCommentsOp;
import org.jetbrains.kotlinx.ktfmt.format.FileType;
import org.jetbrains.kotlinx.ktfmt.format.KotlinCode;
import org.jetbrains.kotlinx.ktfmt.format.KotlinInput;
import org.jetbrains.kotlinx.ktfmt.format.Parser;

/** Interprets an op script against a Kotlin input with gjf's real engine; see ktrs doc tests. */
public class DocScript {
  static Map<String, BreakTag> tags = new HashMap<>();

  static BreakTag tag(String name) {
    return tags.computeIfAbsent(name, n -> new BreakTag());
  }

  static Doc.FillMode mode(String m) {
    switch (m) {
      case "U": return Doc.FillMode.UNIFIED;
      case "I": return Doc.FillMode.INDEPENDENT;
      default: return Doc.FillMode.FORCED;
    }
  }

  static String flat(String f) {
    return f.equals("_") ? "" : f.equals("sp") ? " " : f;
  }

  static String run(String code, int width, String script) {
    tags.clear();
    try {
      CommentsHelper helper = (tok, maxWidth, column0) -> tok.getOriginalText();
      KotlinInput input = new KotlinInput(code, Parser.INSTANCE.parse(KotlinCode.Companion.from(code, FileType.SCRIPT)));
      JavaOutput out = new JavaOutput("\n", input, helper);
      OpsBuilder b = new OpsBuilder(input, out);
      b.markForPartialFormat();
      Iterator<String> it = Arrays.asList(script.trim().split("\\s+")).iterator();
      while (it.hasNext()) {
        String cmd = it.next();
        if (cmd.matches("t\\d*")) {
          int n = cmd.length() == 1 ? 1 : Integer.parseInt(cmd.substring(1));
          for (int i = 0; i < n; i++)
            b.token(b.peekToken().get(), Doc.Token.RealOrImaginary.REAL, Indent.Const.ZERO, Optional.empty());
          continue;
        }
        switch (cmd) {
          case "o": b.open(Indent.Const.make(Integer.parseInt(it.next()), 1)); break;
          case "oif": {
            BreakTag t = tag(it.next());
            Indent th = Indent.Const.make(Integer.parseInt(it.next()), 1);
            Indent el = Indent.Const.make(Integer.parseInt(it.next()), 1);
            b.open(Indent.If.make(t, th, el));
            break;
          }
          case "c": b.close(); break;
          case "tc": b.token(b.peekToken().get(), Doc.Token.RealOrImaginary.REAL,
              Indent.Const.make(Integer.parseInt(it.next()), 1), Optional.empty()); break;
          case "tt": b.token(b.peekToken().get(), Doc.Token.RealOrImaginary.REAL, Indent.Const.ZERO,
              Optional.of(Indent.Const.make(Integer.parseInt(it.next()), 1))); break;
          case "s": b.space(); break;
          case "b": b.breakOp(mode(it.next()), flat(it.next()), Indent.Const.make(Integer.parseInt(it.next()), 1)); break;
          case "bt": {
            Doc.FillMode m = mode(it.next());
            String f = flat(it.next());
            Indent ind = Indent.Const.make(Integer.parseInt(it.next()), 1);
            b.breakOp(m, f, ind, Optional.of(tag(it.next())));
            break;
          }
          case "bl": {
            String w = it.next();
            b.blankLineWanted(w.equals("yes") ? OpsBuilder.BlankLineWanted.YES
                : w.equals("no") ? OpsBuilder.BlankLineWanted.NO
                : w.equals("preserve") ? OpsBuilder.BlankLineWanted.PRESERVE
                : OpsBuilder.BlankLineWanted.conditional(tag(w)));
            break;
          }
          case "g": b.guessToken(it.next()); break;
          case "r": b.token(it.next(), Doc.Token.RealOrImaginary.REAL, Indent.Const.ZERO, Optional.empty()); break;
          case "sync": b.sync(Integer.parseInt(it.next())); break;
          case "fence": b.addAll(FenceCommentsOp.INSTANCE.getAS_LIST()); break;
          case "mark": b.markForPartialFormat(); break;
          default: throw new IllegalArgumentException("bad command " + cmd);
        }
      }
      b.sync(code.length());
      b.drain();
      ImmutableList<Op> ops = b.build();
      Doc doc = new DocBuilder().withOps(ops).build();
      doc.computeBreaks(helper, width, new Doc.State(+0, 0));
      doc.write(out);
      out.flush();
      var ranges = input.characterRangesToTokenRanges(ImmutableList.of(Range.closedOpen(0, code.length())));
      return JavaOutput.applyReplacements(code, out.getFormatReplacements(ranges)).replace('\u0003', ' ');
    } catch (Throwable e) {
      return "ERROR: " + e.getMessage();
    }
  }

  // Sections "=== name\n<width> <script>\n<code>" separated by "\n=== "; writes "=== name\n<output>\n".
  public static void main(String[] args) throws Exception {
    String all = new String(Files.readAllBytes(Paths.get(args[0])), StandardCharsets.UTF_8).replace("\r\n", "\n");
    StringBuilder out = new StringBuilder();
    for (String section : ("\n" + all).split("\n=== ")) {
      if (section.isEmpty()) continue;
      int nl1 = section.indexOf('\n');
      String name = section.substring(0, nl1);
      int nl2 = section.indexOf('\n', nl1 + 1);
      String header = section.substring(nl1 + 1, nl2);
      int sp = header.indexOf(' ');
      String result = run(section.substring(nl2 + 1).replace('⎵', ' '),Integer.parseInt(header.substring(0, sp)), header.substring(sp + 1));
      out.append("=== ").append(name).append('\n').append(result).append('\n');
    }
    Files.write(Paths.get(args[1]), out.toString().getBytes(StandardCharsets.UTF_8));
  }
}
