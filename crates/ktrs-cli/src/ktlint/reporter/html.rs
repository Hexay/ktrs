//! Port of `ktlint-cli-reporter-html` (`HtmlReporter`, MIT, (c) 2019 Matheus Candido).

use crate::ktlint::console::{LINE_SEPARATOR, Printer};
use crate::ktlint::reporter::java_map::JavaConcurrentHashMap;
use crate::ktlint::reporter::{KtlintCliError, ReporterV2, Status, escape_xml_attr_value};

pub struct HtmlReporter {
    out: Printer,
    acc: JavaConcurrentHashMap<Vec<KtlintCliError>>,
    issue_count: usize,
    corrected_count: usize,
}

impl HtmlReporter {
    pub fn new(out: Printer) -> HtmlReporter {
        HtmlReporter { out, acc: JavaConcurrentHashMap::new(), issue_count: 0, corrected_count: 0 }
    }

    fn head(&mut self) {
        let out = &mut self.out;
        out.println("<head>");
        out.print("<link href=\"");
        out.print("https://fonts.googleapis.com/css?family=Source+Code+Pro");
        out.println("\" rel=\"stylesheet\" />");
        out.print(&format!("<meta http-equiv=\"Content-Type\" Content=\"text/html; Charset=UTF-8\">{LINE_SEPARATOR}"));
        out.print(&format!("<style>{LINE_SEPARATOR}"));
        out.print(&format!("body {{{LINE_SEPARATOR}"));
        out.print(&format!("    font-family: 'Source Code Pro', monospace;{LINE_SEPARATOR}"));
        out.print(&format!("}}{LINE_SEPARATOR}"));
        out.print(&format!("h3 {{{LINE_SEPARATOR}"));
        out.print(&format!("    font-size: 12pt;{LINE_SEPARATOR}"));
        out.print("}");
        out.print(&format!("</style>{LINE_SEPARATOR}"));
        out.println("</head>");
    }

    fn body(&mut self) {
        let out = &mut self.out;
        out.println("<body>");
        if !self.acc.is_empty() {
            out.print("<h1>");
            out.print("Overview");
            out.println("</h1>");
            paragraph(out, &format!("Issues found: {}", self.issue_count));
            paragraph(out, &format!("Issues corrected: {}", self.corrected_count));
            for (file, errors) in self.acc.iter() {
                out.print("<h3>");
                out.print(file);
                out.println("</h3>");
                out.println("<ul>");
                for err in errors {
                    out.print("<li>");
                    out.print(&escape_xml_attr_value(&format!(
                        "({}, {}): {}  ({})",
                        err.line, err.col, err.detail, err.rule_id
                    )));
                    out.println("</li>");
                }
                out.println("</ul>");
            }
        } else {
            paragraph(out, "Congratulations, no issues found!");
        }
        out.println("</body>");
    }
}

fn paragraph(out: &mut Printer, text: &str) {
    out.print("<p>");
    out.print(text);
    out.println("</p>");
}

impl ReporterV2 for HtmlReporter {
    fn on_lint_error(&mut self, file: &str, ktlint_cli_error: &KtlintCliError) {
        if ktlint_cli_error.status != Status::FormatIsAutocorrected {
            self.issue_count += 1;
            self.acc.get_or_put(file, Vec::new).push(ktlint_cli_error.clone());
        } else {
            self.corrected_count += 1;
        }
    }

    fn after_all(&mut self) {
        self.out.println("<html>");
        self.head();
        self.body();
        self.out.println("</html>");
    }
}
