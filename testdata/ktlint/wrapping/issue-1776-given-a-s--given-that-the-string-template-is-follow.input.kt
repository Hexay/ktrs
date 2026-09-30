// Max line length marker:                                            #
fun getQueryString(query: QueryRequest): String {
    val q = """
        SELECT *
        FROM table
        WHERE ${query.gameId?.let { "id = ?" } ?: "1 = 1"} OR level = ?
    """
    return q
}