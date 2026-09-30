import com.zak.result.Result.Expected
import com.zak.result.Result.Unexpected
import org.assertj.core.api.Assertions.assertThat

fun test() {
    assertThat(Result.just(1)).isEqualTo(Expected(1))
    assertThat(Result.just(1)).isEqualTo(Result.Expected(1))

    val ex = Exception()
    assertThat(Result.raise(exception)).isEqualTo(Unexpected(ex))
    assertThat(Result.raise(exception)).isEqualTo(Result.Unexpected(exception))
}