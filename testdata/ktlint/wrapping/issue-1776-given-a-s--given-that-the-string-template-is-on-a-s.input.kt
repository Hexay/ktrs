// Max line length marker:                        #
fun getQueryString(query: QueryRequest): String {
    val q = """
        SELECT *
        FROM table
        WHERE 1 = 1
        ${query.gameId?.let { "AND id = ?" } ?: ""}
    """
    return q
}