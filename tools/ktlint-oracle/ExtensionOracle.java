// JVM oracle for crates/ktrs-lint/tests/extension_oracle.rs: one line per node (preorder) of each input file with
// the ASTNodeExtension.kt / IndentConfig.kt values the Rust port must reproduce. Nodes are named by preorder index,
// offsets are UTF-16, `!` marks a thrown exception. The Rust test documents the columns.
// Regenerate (jar: tools/sync-ktlint.sh puts it in tools/ktlint-oracle/lib):
//   javac -cp <jar> -d target/edit-oracle tools/ktlint-oracle/ExtensionOracle.java
//   java -cp "target/edit-oracle;<jar>" ExtensionOracle crates/ktrs-lint/tests/data/extension_sample.kt \
//     testdata/kotlin/psi/{CommentsBinding,FunctionLiterals,annotations}.kt > crates/ktrs-lint/tests/data/extension.jvm.txt
import io.github.ktlint.core.rule.engine.core.api.ASTNodeExtensionKt;
import io.github.ktlint.core.rule.engine.core.api.IndentConfig;
import io.github.ktlint.core.rule.engine.core.api.KtlintKotlinCompiler;
import kotlin.sequences.Sequence;
import org.jetbrains.kotlin.com.intellij.lang.ASTNode;
import org.jetbrains.kotlin.com.intellij.psi.tree.IElementType;
import org.jetbrains.kotlin.lexer.KtTokens;

import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.IdentityHashMap;
import java.util.Iterator;
import java.util.List;
import java.util.function.Supplier;

public class ExtensionOracle {
    static final IdentityHashMap<ASTNode, Integer> INDEX = new IdentityHashMap<>();
    static final IndentConfig SPACES = IndentConfig.Companion.getDEFAULT_INDENT_CONFIG();
    static final IndentConfig TABS = new IndentConfig(IndentConfig.IndentStyle.TAB, 4);

    static void preorder(ASTNode n, List<ASTNode> out) {
        out.add(n);
        for (ASTNode c = n.getFirstChildNode(); c != null; c = c.getTreeNext()) preorder(c, out);
    }

    static String id(ASTNode n) {
        return n == null ? "-" : String.valueOf(INDEX.get(n));
    }

    static String esc(Object o) {
        return String.valueOf(o).replace("\\", "\\\\").replace("\n", "\\n").replace("\t", "\\t");
    }

    static String b(boolean v) {
        return v ? "1" : "0";
    }

    static String safe(Supplier<Object> f) {
        try {
            return esc(f.get());
        } catch (Throwable t) {
            return "!" + t.getClass().getSimpleName();
        }
    }

    static String ids(Sequence<ASTNode> s, int max) {
        StringBuilder sb = new StringBuilder();
        Iterator<ASTNode> it = s.iterator();
        for (int i = 0; i < max && it.hasNext(); i++) sb.append(i == 0 ? "" : ",").append(id(it.next()));
        return sb.toString();
    }

    static int count(Sequence<ASTNode> s) {
        int n = 0;
        for (Iterator<ASTNode> it = s.iterator(); it.hasNext(); it.next()) n++;
        return n;
    }

    static String line(ASTNode n) {
        ASTNode next = n.getTreeNext();
        IElementType ident = KtTokens.IDENTIFIER;
        String indent = ASTNodeExtensionKt.getIndent(n);
        List<String> f = new ArrayList<>();
        f.add(id(n));
        f.add(n.getElementType().toString());
        f.add(n.getStartOffset() + "-" + ASTNodeExtensionKt.getEndOffset(n));
        f.add(id(ASTNodeExtensionKt.getNextLeaf(n)) + "," + id(ASTNodeExtensionKt.getPrevLeaf(n)));
        f.add(id(ASTNodeExtensionKt.getNextCodeLeaf(n)) + "," + id(ASTNodeExtensionKt.getPrevCodeLeaf(n)));
        f.add(id(ASTNodeExtensionKt.getNextCodeSibling(n)) + "," + id(ASTNodeExtensionKt.getPrevCodeSibling(n)));
        f.add(id(ASTNodeExtensionKt.getFirstChildLeafOrSelf(n)) + "," + id(ASTNodeExtensionKt.getLastChildLeafOrSelf(n)));
        f.add(b(ASTNodeExtensionKt.isCode(n)) + b(ASTNodeExtensionKt.isPartOfComment(n)) + b(ASTNodeExtensionKt.isPartOfString(n))
            + b(ASTNodeExtensionKt.isWhiteSpace(n)) + b(ASTNodeExtensionKt.isWhiteSpaceWithNewline(n))
            + b(ASTNodeExtensionKt.isWhiteSpaceWithoutNewline(n)) + b(ASTNodeExtensionKt.isWhiteSpaceWithoutNewlineOrNull(n.getTreePrev()))
            + b(ASTNodeExtensionKt.isRoot(n)) + b(ASTNodeExtensionKt.isLeaf(n)) + b(ASTNodeExtensionKt.isDeclaration(n))
            + b(ASTNodeExtensionKt.isPartOf(n, KtTokens.COMMENTS)));
        f.add(String.valueOf(ASTNodeExtensionKt.getColumn(n)));
        f.add(esc(indent) + "|" + esc(ASTNodeExtensionKt.getIndentWithoutNewlinePrefix(n)));
        f.add(ids(ASTNodeExtensionKt.getLeavesOnLine(n), 3) + "#" + count(ASTNodeExtensionKt.getLeavesOnLine(n)));
        f.add(safe(() -> ASTNodeExtensionKt.getLineLength(ASTNodeExtensionKt.dropTrailingEolComment(ASTNodeExtensionKt.getLeavesOnLine(n)))));
        f.add(b(ASTNodeExtensionKt.hasNoMaxLineLengthSuppression(n)));
        f.add(safe(() -> ASTNodeExtensionKt.isKtAnnotated(n)));
        f.add(id(ASTNodeExtensionKt.findChildByTypeRecursively(n, ident)) + "," + count(ASTNodeExtensionKt.getRecursiveChildren(n)));
        f.add(b(ASTNodeExtensionKt.afterCodeSibling(n, KtTokens.LPAR)) + b(ASTNodeExtensionKt.beforeCodeSibling(n, KtTokens.RPAR))
            + b(ASTNodeExtensionKt.hasModifier(n, KtTokens.PRIVATE_KEYWORD)));
        f.add(ids(ASTNodeExtensionKt.getLeavesForwardsIncludingSelf(n), 2) + "|" + ids(ASTNodeExtensionKt.getLeavesBackwardsIncludingSelf(n), 2));
        if (next != null) {
            f.add(b(ASTNodeExtensionKt.hasNewLineInClosedRange(n, next)) + b(ASTNodeExtensionKt.noNewLineInClosedRange(n, next))
                + b(ASTNodeExtensionKt.noNewLineInOpenRange(n, next)) + count(ASTNodeExtensionKt.leavesInOpenRange(n, next))
                + "," + count(ASTNodeExtensionKt.leavesInClosedRange(n, next)));
        } else {
            f.add("-");
        }
        f.add(safe(() -> SPACES.childIndentOf(n)) + "|" + safe(() -> SPACES.siblingIndentOf(n)) + "|" + safe(() -> SPACES.parentIndentOf(n)));
        f.add(safe(() -> SPACES.indentLevelFrom(indent)) + "|" + safe(() -> SPACES.toNormalizedIndent(indent)) + "|"
            + safe(() -> TABS.toNormalizedIndent(indent)) + "|" + safe(() -> TABS.indentLevelFrom(indent)) + "|"
            + TABS.containsUnexpectedIndentChar(indent) + TABS.indexOfFirstUnexpectedIndentChar(indent));
        return String.join("\t", f);
    }

    public static void main(String[] args) throws Exception {
        StringBuilder out = new StringBuilder();
        for (String file : args) {
            String text = Files.readString(Path.of(file)).replace("\r\n", "\n");
            ASTNode root = KtlintKotlinCompiler.INSTANCE.createPsiFileFromText("File.kt", text).getNode();
            List<ASTNode> nodes = new ArrayList<>();
            preorder(root, nodes);
            INDEX.clear();
            for (int i = 0; i < nodes.size(); i++) INDEX.put(nodes.get(i), i);
            out.append("=== ").append(Path.of(file).getFileName()).append('\n');
            for (ASTNode n : nodes) out.append(line(n)).append('\n');
        }
        System.out.print(out);
    }
}
