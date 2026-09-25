//! Port of `Table.kt`.

use super::kstring::{KChar, KStr, KString, w};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Align {
    Left,
    Right,
    Center,
}

#[derive(Clone, Debug, Default)]
pub struct Row {
    pub cells: Vec<KString>,
}

pub struct Table {
    #[allow(dead_code)]
    columns: usize,
    widths: Vec<usize>,
    rows: Vec<Row>,
    align: Vec<Align>,
    original: Vec<KString>,
}

/// Java `String.format("%-Ns")` / `"%Ns"`: pad with spaces to `width` UTF-16 units.
fn pad(s: &[u16], width: usize, left_justify: bool) -> KString {
    let fill = vec![' ' as u16; width.saturating_sub(s.len())];
    if left_justify { [s, &fill].concat() } else { [&fill, s].concat() }
}

impl Table {
    pub fn original(&self) -> &[KString] {
        &self.original
    }

    /// Format the table. Rows can't be broken, so [max_width] only decides whether to pad cells;
    /// the result may be wider. Upstream default: `Integer.MAX_VALUE`.
    pub fn format(&self, max_width: i32) -> Vec<KString> {
        // +2: "| " in each cell and final " |" on the right
        let table_max_width = 2 + self.widths.iter().map(|it| it + 2).sum::<usize>() as i64;

        let pad_cells = table_max_width <= max_width as i64;
        let mut lines = Vec::new();
        for (i, row) in self.rows.iter().enumerate() {
            let mut sb = KString::new();
            for column in 0..row.cells.len() {
                sb.push('|' as u16);
                if pad_cells {
                    sb.push(' ' as u16);
                }
                let cell = &row.cells[column];
                let width = self.widths[column];
                let s = if self.align[column] == Align::Center && i > 0 {
                    let inner = pad(cell, cell.len() + (width - cell.len()) / 2, false);
                    pad(&inner, width, true)
                } else if self.align[column] == Align::Right && i > 0 {
                    pad(cell, width, false)
                } else {
                    pad(cell, width, true)
                };
                sb.extend_from_slice(&s);
                if pad_cells {
                    sb.push(' ' as u16);
                }
            }
            sb.push('|' as u16);
            lines.push(std::mem::take(&mut sb));

            if i == 0 {
                for column in 0..row.cells.len() {
                    sb.push('|' as u16);
                    let mut width = self.widths[column] as i64;
                    if self.align[column] != Align::Left {
                        width -= 1;
                        if self.align[column] == Align::Center {
                            sb.push(':' as u16);
                            width -= 1;
                        }
                    }
                    if pad_cells {
                        sb.push('-' as u16);
                    }
                    sb.extend(std::iter::repeat_n('-' as u16, width.max(0) as usize));
                    if pad_cells {
                        sb.push('-' as u16);
                    }
                    if self.align[column] != Align::Left {
                        sb.push(':' as u16);
                    }
                }
                sb.push('|' as u16);
                lines.push(std::mem::take(&mut sb));
            }
        }

        lines
    }

    /// If the line at index [start] begins a table, return it and the index of the first line
    /// after the table.
    pub fn get_table(
        lines: &[KString],
        start: usize,
        line_content: &dyn Fn(&[u16]) -> KString,
    ) -> Option<(Table, usize)> {
        if start as i64 > lines.len() as i64 - 2 {
            return None;
        }
        let header_line = line_content(&lines[start]);
        let separator_line = line_content(&lines[start + 1]);
        let bar_count = Self::count_separators(&header_line);
        if !Self::is_header_divider(bar_count, separator_line.trim()) {
            return None;
        }
        let header = Self::get_row(&header_line)?;
        let mut rows = vec![header];

        let mut divider_row = Self::get_row(&separator_line)?;

        let mut i = start + 2;
        while i < lines.len() {
            let line = line_content(&lines[i]);
            if !line.contains_seq(w!("|")) {
                break;
            }
            let Some(row) = Self::get_row(&line) else { break };
            rows.push(row);
            i += 1;
        }

        let all_first_blank = rows
            .iter()
            .chain(std::iter::once(&divider_row))
            .all(|row| row.cells.first().is_some_and(|first| first.is_blank()));
        if all_first_blank {
            for row in rows.iter_mut().chain(std::iter::once(&mut divider_row)) {
                if !row.cells.is_empty() {
                    row.cells.remove(0);
                }
            }
        }

        let columns = divider_row.cells.len();
        let max_columns = rows.iter().map(|it| it.cells.len()).max().expect("NoSuchElementException");
        let mut widths = vec![3usize; max_columns];
        for row in rows.iter_mut() {
            for column in 0..row.cells.len() {
                widths[column] = widths[column].max(row.cells[column].len());
            }
            for _ in row.cells.len()..columns {
                row.cells.push(KString::new());
            }
        }

        let mut align = Vec::new();
        for cell in &divider_row.cells {
            let direction = if cell.ends_with(w!(":")) {
                if cell.starts_with(w!(":-")) { Align::Center } else { Align::Right }
            } else {
                Align::Left
            };
            align.push(direction);
        }
        for _ in align.len()..max_columns {
            align.push(Align::Left);
        }
        let original = lines[start..i].iter().map(|it| line_content(it)).collect();
        let table = Table { columns, widths, rows, align, original };
        Some((table, i))
    }

    /// Returns true if the given String looks like a markdown table header divider.
    fn is_header_divider(bar_count: usize, s: &[u16]) -> bool {
        let mut i = 0;
        let mut count = 0;
        while i < s.len() {
            let c = s[i];
            i += 1;
            if c == '\\' as u16 {
                i += 1;
            } else if c == '|' as u16 {
                count += 1;
            } else if c.is_whitespace() || c == ':' as u16 {
                continue;
            } else if c == '-' as u16
                && (s.starts_with_at(w!("--"), i)
                    || s.starts_with_at(w!("-:"), i)
                    || (i > 1 && s.starts_with_at(w!(":-:"), i - 2))
                    || (i > 1 && s.starts_with_at(w!(":--"), i - 2)))
            {
                while i < s.len() && s[i] == '-' as u16 {
                    i += 1;
                }
            } else {
                return false;
            }
        }

        bar_count == count
    }

    fn get_row(s: &[u16]) -> Option<Row> {
        // Can't just use String.split('|') because that would not handle escaped |'s
        if s.index_of_char('|' as u16, 0) == -1 {
            return None;
        }
        let mut row = Row::default();
        let mut i = 0;
        let mut end = 0;
        while end < s.len() {
            let c = s[end];
            if c == '\\' as u16 {
                end += 1;
            } else if c == '|' as u16 {
                let cell = s[i..end].trim();
                if end > 0 {
                    row.cells.push(cell.trim().to_vec());
                }
                i = end + 1;
            }
            end += 1;
        }
        if end > i {
            // A trailing backslash leaves end == length + 1: StringIndexOutOfBoundsException upstream.
            let cell = s[i..end].trim();
            if !cell.is_empty() {
                row.cells.push(cell.trim().to_vec());
            }
        }

        Some(row)
    }

    fn count_separators(s: &[u16]) -> usize {
        let mut i = 0;
        let mut count = 0;
        while i < s.len() {
            let c = s[i];
            if c == '|' as u16 {
                count += 1;
            } else if c == '\\' as u16 {
                i += 1;
            }
            i += 1;
        }
        count
    }
}
