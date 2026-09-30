// Max line length marker:                               #
fun getQueryString1(query: QueryRequest): String {
    val q = """
        SELECT *
        FROM table
        WHERE ${query.gameId?.let { "id = ?" } ?: "1 = 1"}
    """
    return q
}