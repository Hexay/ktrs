package io.github.hexay.ktrs.maven.ktlint;

import java.util.Map;
import org.apache.maven.plugin.logging.Log;

public class MavenLogReporterProvider {
    public final String id = "maven";

    public MavenLogReporter get(Log log, Map<String, String> opt) {
        return new MavenLogReporter(log, emptyOrTrue(opt.get("verbose")), emptyOrTrue(opt.get("group_by_file")),
                emptyOrTrue(opt.get("pad")));
    }

    private static boolean emptyOrTrue(String value) {
        return "".equals(value) || "true".equals(value);
    }
}
