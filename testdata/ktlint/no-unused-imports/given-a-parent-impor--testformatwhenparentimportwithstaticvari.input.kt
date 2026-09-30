package org.tw.project
import org.repository.RepositoryPolicy
import org.repository.RepositoryPolicy.CHECKSUM_POLICY_IGNORE
import org.mockito.Mockito
import org.mockito.Mockito.withSettings
fun foo() {
        Mockito.mock(String::class.java, Mockito.withSettings().defaultAnswer {  })
}
fun main() {
       val a = CHECKSUM_POLICY_IGNORE
}