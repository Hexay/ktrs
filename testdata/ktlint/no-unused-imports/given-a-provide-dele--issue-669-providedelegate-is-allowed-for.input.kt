import com.github.ajalt.clikt.parameters.groups.provideDelegate

fun main() {
    private val old by argument("OLD", help = "Old input file.")
      .path(exists = true, folderOkay = false, readable = true, fileSystem = inputFs)
}