package io.github.hexay.ktrs.maven.ktlint.internal;

import java.util.Comparator;
import java.util.List;
import java.util.Map;
import java.util.ResourceBundle;
import java.util.TreeMap;
import java.util.stream.Collectors;
import org.apache.maven.doxia.sink.Sink;
import org.apache.maven.doxia.sink.SinkEventAttributes;
import org.apache.maven.doxia.sink.impl.SinkEventAttributeSet;

/** gantsign's report page, with the Sink calls its {@code doxia-sink-api-ktx} DSL makes. */
public final class KtlintReportGenerator {
    private final Sink sink;
    private final ResourceBundle bundle;
    private final String title;

    public KtlintReportGenerator(Sink sink, ResourceBundle bundle) {
        this.sink = sink;
        this.bundle = bundle;
        this.title = get(bundle, "report.ktlint.title");
    }

    /** {@code ResourceBundle.get}: the key itself when the bundle lacks it. */
    public static String get(ResourceBundle bundle, String key) {
        return bundle.containsKey(key) ? bundle.getString(key) : key;
    }

    public void generatorReport(CheckResults results) {
        sink.head(attrs());
        sink.title(attrs());
        sink.text(title);
        sink.title_();
        sink.head_();
        sink.body(attrs());

        section(1);
        sectionTitle(1, title);
        sink.paragraph(attrs());
        sink.text(get(bundle, "report.ktlint.ktlintlink") + " ");
        sink.link("https://github.com/pinterest/ktlint", attrs());
        sink.text("ktlint");
        sink.link_();
        // Upstream appends the rule engine jar's Implementation-Version, which ktlint's jars don't set.
        sink.text(".");
        sink.paragraph_();
        sink.section_(1);

        section(1);
        sectionTitle(1, get(bundle, "report.ktlint.summary"));
        sink.table(attrs());
        sink.tableRows(new int[0], false);
        row(true, get(bundle, "report.ktlint.files"), get(bundle, "report.ktlint.errors"));
        row(false, String.valueOf(results.fileCount), String.valueOf(results.errors.size()));
        sink.tableRows_();
        sink.table_();
        sink.section_(1);

        Map<String, List<FileLintError>> errorsByFile = results.errors.stream()
                .collect(Collectors.groupingBy(it -> it.file, TreeMap::new, Collectors.toList()));

        if (!errorsByFile.isEmpty()) {
            section(1);
            sectionTitle(1, get(bundle, "report.ktlint.files"));
            sink.table(attrs());
            sink.tableRows(new int[0], false);
            row(true, get(bundle, "report.ktlint.file"), get(bundle, "report.ktlint.errors"));
            errorsByFile.forEach((file, errors) -> {
                sink.tableRow(attrs());
                sink.tableCell(attrs());
                sink.link("#" + file.replace('/', '.'), attrs());
                sink.text(file);
                sink.link_();
                sink.tableCell_();
                cell(false, String.valueOf(errors.size()));
                sink.tableRow_();
            });
            sink.tableRows_();
            sink.table_();
            sink.section_(1);

            section(1);
            sectionTitle(1, get(bundle, "report.ktlint.details"));
            errorsByFile.forEach((file, errors) -> {
                List<FileLintError> sortedErrors = errors.stream()
                        .sorted(Comparator.comparingInt(FileLintError::getLine)
                                .thenComparingInt(FileLintError::getCol)
                                .thenComparing(FileLintError::getRuleId))
                        .collect(Collectors.toList());
                SinkEventAttributeSet id = new SinkEventAttributeSet();
                id.addAttribute(SinkEventAttributes.ID, file.replace('/', '.'));
                sink.section(2, id);
                sectionTitle(2, file);
                sink.section_(2);
                sink.table(attrs());
                sink.tableRows(new int[0], false);
                row(true, get(bundle, "report.ktlint.detail"), get(bundle, "report.ktlint.ruleId"),
                        get(bundle, "report.ktlint.line"));
                for (FileLintError error : sortedErrors) {
                    row(false, error.detail, error.ruleId, String.valueOf(error.line));
                }
                sink.tableRows_();
                sink.table_();
            });
            sink.section_(1);
        }
        sink.body_();
    }

    private static SinkEventAttributeSet attrs() {
        return new SinkEventAttributeSet();
    }

    private void section(int level) {
        sink.section(level, attrs());
    }

    private void sectionTitle(int level, String text) {
        sink.sectionTitle(level, attrs());
        sink.text(text);
        sink.sectionTitle_(level);
    }

    private void row(boolean header, String... cells) {
        sink.tableRow(attrs());
        for (String text : cells) cell(header, text);
        sink.tableRow_();
    }

    private void cell(boolean header, String text) {
        if (header) {
            sink.tableHeaderCell(attrs());
            sink.text(text);
            sink.tableHeaderCell_();
        } else {
            sink.tableCell(attrs());
            sink.text(text);
            sink.tableCell_();
        }
    }
}
