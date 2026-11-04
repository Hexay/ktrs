/** Lone punctuation code spans: `,` and `.` and `;` and `:` and `(` and `)` and `[` and `]`. */
val punct = 0

/** Separator `,` span, then an `a, b` span whose comma is interior. */
val comma = 1

/** Operator spans `->` and `::` and a bare `=` span. */
val ops = 2

/** A double-backtick ``a ` b`` span around a literal backtick. */
val nested = 3
