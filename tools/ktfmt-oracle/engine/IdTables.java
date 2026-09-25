import java.util.function.IntPredicate;
public class IdTables {
  static void table(String name, IntPredicate p) {
    StringBuilder sb = new StringBuilder("pub(super) static " + name + ": &[(u32, u32)] = &[\n");
    int n = 0; int cp = 0x80;
    StringBuilder line = new StringBuilder("   ");
    while (cp <= 0x10FFFF) {
      if (!p.test(cp)) { cp++; continue; }
      int lo = cp; while (cp + 1 <= 0x10FFFF && p.test(cp + 1)) cp++;
      String item = String.format(" (0x%x, 0x%x),", lo, cp);
      if (line.length() + item.length() > 119) { sb.append(line).append('\n'); line = new StringBuilder("   "); }
      line.append(item); n++; cp++;
    }
    sb.append(line).append("\n];\n");
    System.out.print(sb);
    System.err.println(name + " " + n);
  }
  public static void main(String[] a) {
    table("JAVA_IDENTIFIER_START", Character::isJavaIdentifierStart);
    table("JAVA_IDENTIFIER_PART", Character::isJavaIdentifierPart);
  }
}
