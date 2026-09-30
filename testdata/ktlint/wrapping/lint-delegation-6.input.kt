data class Shortcut(val id: String, val url: String)

object Someclass : List<Shortcut> by listOf(
    Shortcut(
        id = "1",
        url = "url"
    ),
    Shortcut(
        id = "2",
        url = "asd"
    ),
    Shortcut(
        id = "3",
        url = "TV"
    )
)