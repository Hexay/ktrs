import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.List;
import java.util.stream.Collectors;
import java.util.stream.Stream;

import org.jetbrains.kotlin.cli.FrontendConfigurationKeysKt;
import org.jetbrains.kotlin.cli.common.messages.MessageCollector;
import org.jetbrains.kotlin.cli.jvm.compiler.EnvironmentConfigFiles;
import org.jetbrains.kotlin.cli.jvm.compiler.KotlinCoreEnvironment;
import org.jetbrains.kotlin.com.intellij.openapi.project.Project;
import org.jetbrains.kotlin.com.intellij.openapi.util.Disposer;
import org.jetbrains.kotlin.com.intellij.psi.PsiElement;
import org.jetbrains.kotlin.compiler.plugin.CompilerPluginRegistrar;
import org.jetbrains.kotlin.config.CommonConfigurationKeys;
import org.jetbrains.kotlin.config.CompilerConfiguration;
import org.jetbrains.kotlin.psi.KtFile;
import org.jetbrains.kotlin.psi.KtPsiFactory;

/**
 * Differential oracle for crates/ktrs_psi: for every PSI element prints the class, type tests, visitor dispatch
 * chain, navigation, and every accessor ktfmt uses; then the full KtTreeVisitorVoid walk.
 *
 *   PsiAccessors one <file> [--fixture] [--script]          print the report
 *   PsiAccessors hashes <dir> [--fixture] [--script]        "<rel path>\t<fnv1a64 of report>" per file
 *   PsiAccessors dump <dir> <out-dir> [--fixture] [--script] report per file into out-dir/<rel>.txt
 *
 * --fixture: input is CRLF->LF with trailing newlines stripped and only files with a sibling .txt are used
 * (the compiler's parser-test convention); otherwise CRLF->LF only. --script parses every file as "temp.kts",
 * like ktfmt does.
 */
public final class PsiAccessors {
    public static void main(String[] args) throws Exception {
        if (args.length < 2) usage();
        boolean fixture = false, script = false;
        List<String> positional = new java.util.ArrayList<>();
        for (String a : args) {
            if (a.equals("--fixture")) fixture = true;
            else if (a.equals("--script")) script = true;
            else positional.add(a);
        }
        KtPsiFactory factory = factory();
        switch (positional.get(0)) {
            case "one":
                System.out.print(report(factory, Paths.get(positional.get(1)), fixture, script));
                break;
            case "hashes": {
                Path dir = Paths.get(positional.get(1));
                StringBuilder sb = new StringBuilder();
                for (Path f : files(dir, fixture)) {
                    sb.append(rel(dir, f)).append('\t')
                      .append(String.format("%016x", Fmt.fnv1a64(report(factory, f, fixture, script)))).append('\n');
                }
                System.out.print(sb);
                break;
            }
            case "dump": {
                Path dir = Paths.get(positional.get(1)), out = Paths.get(positional.get(2));
                for (Path f : files(dir, fixture)) {
                    Path target = out.resolve(rel(dir, f) + ".txt");
                    Files.createDirectories(target.getParent());
                    Files.write(target, report(factory, f, fixture, script).getBytes(StandardCharsets.UTF_8));
                }
                break;
            }
            default:
                usage();
        }
        System.exit(0);
    }

    private static void usage() {
        System.err.println("usage: PsiAccessors one <file> | hashes <dir> | dump <dir> <out-dir>  [--fixture] [--script]");
        System.exit(2);
    }

    static String report(KtPsiFactory factory, Path file, boolean fixture, boolean script) throws IOException {
        String text = new String(Files.readAllBytes(file), StandardCharsets.UTF_8).replace("\r\n", "\n");
        if (fixture) {
            int end = text.length();
            while (end > 0 && text.charAt(end - 1) == '\n') end--;
            text = text.substring(0, end);
        }
        KtFile kt = factory.createFile(script ? "temp.kts" : file.getFileName().toString(), text);
        StringBuilder sb = new StringBuilder();
        try {
            describeTree(kt, sb);
            sb.append("walk\n");
            Recorder walk = new Recorder(true);
            kt.accept(walk);
            sb.append(walk.out);
        } catch (Throwable t) {
            sb.append("CRASH ").append(t).append('\n');
        }
        return sb.toString();
    }

    private static void describeTree(PsiElement root, StringBuilder sb) {
        java.util.ArrayDeque<PsiElement> stack = new java.util.ArrayDeque<>();
        stack.push(root);
        while (!stack.isEmpty()) {
            PsiElement e = stack.pop();
            Base.describe(e, sb);
            java.util.ArrayList<PsiElement> children = new java.util.ArrayList<>();
            for (PsiElement c = e.getFirstChild(); c != null; c = c.getNextSibling()) children.add(c);
            for (int i = children.size() - 1; i >= 0; i--) stack.push(children.get(i));
        }
    }

    private static String rel(Path dir, Path file) {
        return dir.relativize(file).toString().replace('\\', '/');
    }

    private static List<Path> files(Path dir, boolean fixture) throws IOException {
        try (Stream<Path> walk = Files.walk(dir)) {
            return walk.filter(p -> {
                String n = p.getFileName().toString();
                if (!Files.isRegularFile(p) || !(n.endsWith(".kt") || n.endsWith(".kts"))) return false;
                if (!fixture) return true;
                return Files.isRegularFile(p.resolveSibling(n.substring(0, n.lastIndexOf('.')) + ".txt"));
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
}
