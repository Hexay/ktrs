import java.util.List;
import java.util.function.Supplier;

import org.jetbrains.kotlin.com.intellij.psi.PsiElement;
import org.jetbrains.kotlin.com.intellij.psi.PsiFile;
import org.jetbrains.kotlin.com.intellij.psi.tree.IElementType;

/** Canonical value rendering shared by every accessor line; crates/ktrs-psi/examples/psi_accessors mirrors it. */
final class Fmt {
    private Fmt() {}

    static String ref(PsiElement e) {
        if (e == null) return "null";
        String kind = e instanceof PsiFile ? "FILE" : e.getNode().getElementType().toString();
        return kind + "@" + e.getTextRange().getStartOffset() + ".." + e.getTextRange().getEndOffset();
    }

    static String kind(IElementType t) {
        return t == null ? "null" : t.toString();
    }

    static String refs(List<? extends PsiElement> list) {
        StringBuilder sb = new StringBuilder("[");
        for (int i = 0; i < list.size(); i++) {
            if (i > 0) sb.append(", ");
            sb.append(ref(list.get(i)));
        }
        return sb.append(']').toString();
    }

    static String refs(PsiElement[] array) {
        return refs(java.util.Arrays.asList(array));
    }

    static String str(String s) {
        return s == null ? "null" : '"' + s.replace("\\", "\\\\").replace("\n", "\\n").replace("\r", "\\r") + '"';
    }

    static String strs(List<String> list) {
        StringBuilder sb = new StringBuilder("[");
        for (int i = 0; i < list.size(); i++) {
            if (i > 0) sb.append(", ");
            sb.append(str(list.get(i)));
        }
        return sb.append(']').toString();
    }

    /** Evaluates an accessor; any throw renders as "!" (the Rust side returns None/Err there). */
    static String safe(Supplier<String> accessor) {
        try {
            return accessor.get();
        } catch (Throwable t) {
            return "!";
        }
    }

    static long fnv1a64(String s) {
        long hash = 0xcbf29ce484222325L;
        for (byte b : s.getBytes(java.nio.charset.StandardCharsets.UTF_8)) {
            hash ^= (b & 0xff);
            hash *= 0x100000001b3L;
        }
        return hash;
    }
}
