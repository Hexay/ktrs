// JVM oracle for crates/ktrs-ast/tests/edit_oracle.rs: runs each case of crates/ktrs-ast/tests/data/edit_cases.tsv
// (name, op, element type, nth in preorder, text with \n escapes) on the ktlint fat jar's IntelliJ tree and prints
// `=== name` then DebugUtil.psiToString(file, true, false).
//   op delete: node.getPsi().delete()        op remove: CodeEditUtil.removeChild(node.getTreeParent(), node)
// Regenerate (jar: tools/sync-ktlint.sh puts it in tools/ktlint-oracle/lib):
//   javac -cp <jar> -d target/edit-oracle tools/ktlint-oracle/EditOracle.java
//   java -cp "target/edit-oracle;<jar>" EditOracle crates/ktrs-ast/tests/data/edit_cases.tsv > crates/ktrs-ast/tests/data/edit_cases.jvm.txt
import io.github.ktlint.core.rule.engine.core.api.KtlintKotlinCompiler;
import org.jetbrains.kotlin.com.intellij.lang.ASTNode;
import org.jetbrains.kotlin.com.intellij.psi.impl.DebugUtil;
import org.jetbrains.kotlin.com.intellij.psi.impl.source.codeStyle.CodeEditUtil;

import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;

public class EditOracle {
    static void preorder(ASTNode n, List<ASTNode> out) {
        out.add(n);
        for (ASTNode c = n.getFirstChildNode(); c != null; c = c.getTreeNext()) preorder(c, out);
    }

    static ASTNode find(ASTNode root, String type, int nth) {
        List<ASTNode> all = new ArrayList<>();
        preorder(root, all);
        int i = 0;
        for (ASTNode n : all) if (n.getElementType().toString().equals(type) && i++ == nth) return n;
        throw new IllegalStateException("no " + type + " #" + nth);
    }

    public static void main(String[] args) throws Exception {
        StringBuilder out = new StringBuilder();
        for (String line : Files.readAllLines(Path.of(args[0]))) {
            if (line.isBlank()) continue;
            String[] c = line.split("\t", 5);
            String text = c[4].replace("\\n", "\n");
            ASTNode root = KtlintKotlinCompiler.INSTANCE.createPsiFileFromText("File.kt", text).getNode();
            ASTNode node = find(root, c[2], Integer.parseInt(c[3]));
            out.append("=== ").append(c[0]).append('\n');
            try {
                if (c[1].equals("delete")) node.getPsi().delete();
                else CodeEditUtil.removeChild(node.getTreeParent(), node);
                out.append(DebugUtil.psiToString(root.getPsi(), true, false));
            } catch (Throwable t) {
                out.append("THROWS ").append(t).append('\n');
            }
        }
        System.out.print(out);
    }
}
