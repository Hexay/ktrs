import java.io.IOException;
import java.lang.reflect.Field;
import java.lang.reflect.Modifier;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.Collections;
import java.util.IdentityHashMap;
import java.util.List;
import java.util.Set;
import java.util.stream.Collectors;
import java.util.stream.Stream;

import org.jetbrains.kotlin.cli.FrontendConfigurationKeysKt;
import org.jetbrains.kotlin.cli.common.messages.MessageCollector;
import org.jetbrains.kotlin.compiler.plugin.CompilerPluginRegistrar;
import org.jetbrains.kotlin.cli.jvm.compiler.EnvironmentConfigFiles;
import org.jetbrains.kotlin.cli.jvm.compiler.KotlinCoreEnvironment;
import org.jetbrains.kotlin.com.intellij.lang.ASTNode;
import org.jetbrains.kotlin.com.intellij.openapi.project.Project;
import org.jetbrains.kotlin.com.intellij.openapi.util.Disposer;
import org.jetbrains.kotlin.com.intellij.psi.TokenType;
import org.jetbrains.kotlin.com.intellij.psi.impl.DebugUtil;
import org.jetbrains.kotlin.com.intellij.psi.tree.IElementType;
import org.jetbrains.kotlin.com.intellij.psi.tree.ILazyParseableElementTypeBase;
import org.jetbrains.kotlin.kdoc.lexer.KDocTokens;
import org.jetbrains.kotlin.lexer.KtTokens;
import org.jetbrains.kotlin.config.CommonConfigurationKeys;
import org.jetbrains.kotlin.config.CompilerConfiguration;
import org.jetbrains.kotlin.lexer.KtKeywordToken;
import org.jetbrains.kotlin.psi.KtFile;
import org.jetbrains.kotlin.psi.KtPsiFactory;

/**
 * Reference oracle: prints Kotlin PSI trees in the DebugUtil.psiToString format used by the
 * compiler's own parser fixtures, plus the table of element types the Rust kinds are generated from.
 *
 *   PsiDump one <file.kt>            dump one file to stdout
 *   PsiDump tree <in-dir> <out-dir>  dump every .kt/.kts under in-dir to out-dir/<rel>.txt
 *   PsiDump kinds                    element-type table as TSV (owner, field, debugName, class, keyword, soft)
 *   PsiDump bench <dir> <warmup> <reps>  single-threaded parse throughput
 *
 * Input CRLF is normalized to LF, matching IntelliJ's file loading and ktfmt/ktlint.
 */
public final class PsiDump {
    private static final String[] KIND_OWNERS = {
        "org.jetbrains.kotlin.lexer.KtTokens",
        "org.jetbrains.kotlin.KtNodeTypes",
        "org.jetbrains.kotlin.psi.stubs.elements.KtStubElementTypes",
        "org.jetbrains.kotlin.kdoc.lexer.KDocTokens",
        "org.jetbrains.kotlin.kdoc.parser.KDocElementTypes",
    };

    public static void main(String[] args) throws Exception {
        if (args.length == 0) usage();
        switch (args[0]) {
            case "kinds": printKinds(); break;
            case "one": if (args.length != 2) usage(); System.out.print(dump(factory(), Paths.get(args[1]))); break;
            case "tree": if (args.length != 3) usage(); dumpTree(Paths.get(args[1]), Paths.get(args[2])); break;
            case "bench": if (args.length != 4) usage();
                bench(Paths.get(args[1]), Integer.parseInt(args[2]), Integer.parseInt(args[3])); break;
            default: usage();
        }
        System.exit(0);
    }

    private static void usage() {
        System.err.println("usage: PsiDump one <file> | tree <in-dir> <out-dir> | kinds | bench <dir> <warmup> <reps>");
        System.exit(2);
    }

    /**
     * Single-threaded parse throughput, comparable to ktrs's `bench` example: createFile plus a full AST walk,
     * which forces the lazily parsed BLOCK/LAMBDA_EXPRESSION/KDoc nodes. Reports the best rep by thread CPU time.
     */
    private static void bench(Path dir, int warmup, int reps) throws IOException {
        KtPsiFactory factory = factory();
        List<String[]> inputs = new java.util.ArrayList<>();
        long bytes = 0;
        for (Path f : kotlinFiles(dir)) {
            byte[] raw = Files.readAllBytes(f);
            bytes += raw.length;
            inputs.add(new String[] {f.getFileName().toString(), new String(raw, StandardCharsets.UTF_8).replace("\r\n", "\n")});
        }
        java.lang.management.ThreadMXBean mx = java.lang.management.ManagementFactory.getThreadMXBean();
        long best = Long.MAX_VALUE, nodes = 0;
        for (int rep = 0; rep < warmup + reps; rep++) {
            long start = mx.getCurrentThreadCpuTime();
            for (String[] in : inputs) nodes += walk(factory.createFile(in[0], in[1]).getNode());
            long cpu = mx.getCurrentThreadCpuTime() - start;
            System.err.printf("%s %d: %.1f MB/s%n", rep < warmup ? "warmup" : "rep", rep, bytes / 1e6 / (cpu / 1e9));
            if (rep >= warmup) best = Math.min(best, cpu);
        }
        System.out.printf("jvm: %d files, %.1f MB, best %.2f CPU-s = %.1f MB/s (nodes %d)%n",
                inputs.size(), bytes / 1e6, best / 1e9, bytes / 1e6 / (best / 1e9), nodes);
    }

    private static long walk(ASTNode root) {
        long count = 0;
        java.util.ArrayDeque<ASTNode> stack = new java.util.ArrayDeque<>();
        stack.push(root);
        while (!stack.isEmpty()) {
            ASTNode node = stack.pop();
            count++;
            for (ASTNode c = node.getFirstChildNode(); c != null; c = c.getTreeNext()) stack.push(c);
        }
        return count;
    }

    private static List<Path> kotlinFiles(Path dir) throws IOException {
        try (Stream<Path> walk = Files.walk(dir)) {
            return walk.filter(p -> {
                String n = p.getFileName().toString();
                return Files.isRegularFile(p) && (n.endsWith(".kt") || n.endsWith(".kts"));
            }).sorted().collect(Collectors.toList());
        }
    }

    private static KtPsiFactory factory() {
        CompilerConfiguration config = new CompilerConfiguration();
        config.put(CommonConfigurationKeys.MESSAGE_COLLECTOR_KEY, MessageCollector.Companion.getNONE());
        FrontendConfigurationKeysKt.setExtensionsStorage(config, new CompilerPluginRegistrar.ExtensionStorage());
        Project project = KotlinCoreEnvironment.createForProduction(
                Disposer.newDisposable(), config, EnvironmentConfigFiles.JVM_CONFIG_FILES).getProject();
        return new KtPsiFactory(project, false);
    }

    private static String dump(KtPsiFactory factory, Path file) throws IOException {
        String text = new String(Files.readAllBytes(file), StandardCharsets.UTF_8).replace("\r\n", "\n");
        KtFile kt = factory.createFile(file.getFileName().toString(), text);
        return DebugUtil.psiToString(kt, /* showWhitespaces = */ true, /* showRanges = */ false);
    }

    private static void dumpTree(Path in, Path out) throws IOException {
        KtPsiFactory factory = factory();
        List<Path> files = kotlinFiles(in);
        int failed = 0;
        for (Path f : files) {
            Path target = out.resolve(in.relativize(f).toString() + ".txt");
            try {
                Files.createDirectories(target.getParent());
                Files.write(target, dump(factory, f).getBytes(StandardCharsets.UTF_8));
            } catch (Exception | StackOverflowError e) {
                failed++;
                System.err.println("FAIL " + f + ": " + e);
            }
        }
        System.err.println("dumped " + (files.size() - failed) + "/" + files.size() + " files");
    }

    private static void printKinds() throws Exception {
        StringBuilder sb = new StringBuilder("owner\tfield\tdebug_name\tclass\tshape\tkeyword\tsoft\n");
        Set<IElementType> seen = Collections.newSetFromMap(new IdentityHashMap<>());
        for (Class<?> owner : kindOwners()) {
            for (Field f : owner.getFields()) {
                if (!Modifier.isStatic(f.getModifiers()) || !IElementType.class.isAssignableFrom(f.getType())) continue;
                IElementType t = (IElementType) f.get(null);
                if (t == null || !seen.add(t)) continue;
                String keyword = "", soft = "";
                if (t instanceof KtKeywordToken) {
                    keyword = ((KtKeywordToken) t).getValue();
                    soft = String.valueOf(((KtKeywordToken) t).isSoft());
                }
                sb.append(owner.getSimpleName()).append('\t').append(f.getName()).append('\t')
                  .append(escape(t.toString())).append('\t').append(t.getClass().getSimpleName()).append('\t')
                  .append(shape(owner, t)).append('\t').append(keyword).append('\t').append(soft).append('\n');
            }
        }
        System.out.print(sb);
    }

    private static Class<?>[] kindOwners() throws ClassNotFoundException {
        Class<?>[] owners = new Class<?>[KIND_OWNERS.length + 1];
        for (int i = 0; i < KIND_OWNERS.length; i++) owners[i] = Class.forName(KIND_OWNERS[i]);
        owners[KIND_OWNERS.length] = TokenType.class;
        return owners;
    }

    /** token = lexer leaf; lazy = token-declared type the parser expands into a subtree (KDoc); node = composite. */
    private static String shape(Class<?> owner, IElementType t) {
        if (t instanceof ILazyParseableElementTypeBase) return "lazy";
        boolean tokenOwner = owner == KtTokens.class || owner == KDocTokens.class || owner == TokenType.class;
        return tokenOwner && t != TokenType.ERROR_ELEMENT ? "token" : "node";
    }

    private static String escape(String s) {
        return s.replace("\\", "\\\\").replace("\t", "\\t").replace("\n", "\\n");
    }
}
