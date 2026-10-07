package io.github.hexay.ktrs.maven.ktlint;

import io.github.hexay.ktrs.maven.ktlint.internal.KtlintCliError;
import io.github.hexay.ktrs.maven.ktlint.internal.ReporterV2;
import java.io.File;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.concurrent.ConcurrentHashMap;
import org.apache.maven.plugin.logging.Log;
import org.apache.maven.shared.utils.logging.MessageBuilder;
import org.apache.maven.shared.utils.logging.MessageUtils;

public class MavenLogReporter implements ReporterV2 {
    public static final String NAME = "maven";

    private final Log log;
    private final boolean verbose;
    private final boolean groupByFile;
    private final boolean pad;
    private final Map<String, List<KtlintCliError>> acc = new ConcurrentHashMap<>();

    public MavenLogReporter(Log log, boolean verbose, boolean groupByFile, boolean pad) {
        this.log = log;
        this.verbose = verbose;
        this.groupByFile = groupByFile;
        this.pad = pad;
    }

    @Override
    public void onLintError(String file, KtlintCliError ktlintCliError) {
        int line = ktlintCliError.line;
        int col = ktlintCliError.col;
        String ruleId = ktlintCliError.ruleId;
        String detail = ktlintCliError.detail;

        if (groupByFile) {
            acc.computeIfAbsent(file, k -> new ArrayList<>()).add(ktlintCliError);
            return;
        }

        MessageBuilder buf = MessageUtils.buffer()
                .a(dir(file))
                .strong(name(file))
                .a(":")
                .strong(line)
                .a(pad(":" + col + ":", 4))
                .a(" ")
                .failure(detail);
        if (verbose) {
            buf.a(" (" + ruleId + ")");
        }
        log.error(buf.toString());
    }

    @Override
    public void after(String file) {
        if (!groupByFile) return;

        List<KtlintCliError> errList = acc.get(file);
        if (errList == null) return;

        log.error(MessageUtils.buffer().a(dir(file)).strong(name(file)).toString());

        for (KtlintCliError err : errList) {
            MessageBuilder buf = MessageUtils.buffer()
                    .a(" ")
                    .strong(err.line)
                    .a(pad(":" + err.col, 4))
                    .a(" ")
                    .failure(err.detail);
            if (verbose) {
                buf.a(" (" + err.ruleId + ")");
            }

            log.error(buf.toString());
        }
    }

    private String pad(String s, int length) {
        if (!pad || s.length() >= length) return s;
        return s + " ".repeat(length - s.length());
    }

    // Kotlin's substringBeforeLast: the whole path when it has no separator ("a.kts" -> "a.kts\").
    private static String dir(String file) {
        int i = file.lastIndexOf(File.separator);
        return (i < 0 ? file : file.substring(0, i)) + File.separator;
    }

    private static String name(String file) {
        return file.substring(file.lastIndexOf(File.separator) + 1);
    }
}
