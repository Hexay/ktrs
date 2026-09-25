import java.nio.charset.StandardCharsets;
import java.nio.file.*;
import java.util.Arrays;
import java.util.List;
import java.util.stream.Collectors;
import java.util.stream.Stream;

import org.jetbrains.kotlin.com.intellij.lexer.Lexer;
import org.jetbrains.kotlin.kdoc.lexer.KDocLexer;
import org.jetbrains.kotlin.lexer.KotlinLexer;

// Oracle for tests/oracle.rs. Usage: LexDump kotlin|kdoc <dir> <ext>...
// For every file under <dir> ending in one of <ext> writes <file>.<mode>.tok: "kind start end" per token
// (UTF-16 offsets). A `.hexlines` file holds one hex-encoded UTF-8 case per line; each case's tokens
// are preceded by a "# <line index>" line.
public class LexDump {
    public static void main(String[] args) throws Exception {
        String mode = args[0];
        List<String> exts = Arrays.asList(args).subList(2, args.length);
        List<Path> inputs;
        try (Stream<Path> files = Files.walk(Paths.get(args[1]))) {
            inputs = files.filter(p -> exts.stream().anyMatch(e -> p.toString().endsWith(e))).collect(Collectors.toList());
        }
        for (Path in : inputs) {
            StringBuilder sb = new StringBuilder();
            if (in.toString().endsWith(".hexlines")) {
                List<String> lines = Files.readAllLines(in, StandardCharsets.UTF_8);
                for (int i = 0; i < lines.size(); i++) {
                    sb.append("# ").append(i).append('\n');
                    dump(mode, new String(unhex(lines.get(i)), StandardCharsets.UTF_8), sb);
                }
            } else {
                dump(mode, new String(Files.readAllBytes(in), StandardCharsets.UTF_8), sb);
            }
            Path out = in.resolveSibling(in.getFileName() + "." + mode + ".tok");
            Files.write(out, sb.toString().getBytes(StandardCharsets.UTF_8));
        }
    }

    private static byte[] unhex(String hex) {
        byte[] bytes = new byte[hex.length() / 2];
        for (int i = 0; i < bytes.length; i++) {
            bytes[i] = (byte) Integer.parseInt(hex.substring(2 * i, 2 * i + 2), 16);
        }
        return bytes;
    }

    private static void dump(String mode, String text, StringBuilder sb) {
        Lexer lexer = mode.equals("kdoc") ? new KDocLexer() : new KotlinLexer();
        try {
            lexer.start(text);
            while (lexer.getTokenType() != null) {
                sb.append(lexer.getTokenType()).append(' ').append(lexer.getTokenStart()).append(' ').append(lexer.getTokenEnd()).append('\n');
                lexer.advance();
            }
        } catch (Throwable t) {
            sb.append("EXCEPTION ").append(t.getClass().getSimpleName()).append('\n');
        }
    }
}
