import java.nio.file.Files
import java.nio.file.Paths
import java.util.Objects

object TestData {
    fun loadString(dataset: String): String {
        val path = Paths.get(Objects.requireNonNull(TestData::class.java.getResource(dataset)).toURI())
        return String(Files.readAllBytes(path))
    }
}