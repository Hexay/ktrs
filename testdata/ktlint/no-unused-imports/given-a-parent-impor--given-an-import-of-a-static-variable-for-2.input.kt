import org.repository.RepositoryPolicy
import org.repository.RepositoryPolicy.CHECKSUM_POLICY_IGNORE
fun main() {
        RepositoryPolicy(
        false, "trial",
        CHECKSUM_POLICY_IGNORE
    )
}