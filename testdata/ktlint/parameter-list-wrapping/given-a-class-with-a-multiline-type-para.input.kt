// https://github.com/ktlint/ktlint/issues/921
class ComposableLambda<
    P1, P2, P3, P4, P5, P6, P7, P8, P9, P10, P11, P12, P13, P14, P15, P16,
    P17, P18, R>(
    val key: Int,
    private val tracked: Boolean,
    private val sourceInformation: String?
)
// https://github.com/ktlint/ktlint/issues/938
class GenericTypeWithALongLongALong1
class GenericTypeWithALongLongALong2
class GenericTypeWithALongLongALong3
class ViewModelWithALongLongLongLongLongLongLongLongName3<
    A : GenericTypeWithALongLongALong1,
    B : GenericTypeWithALongLongALong2,
    C : GenericTypeWithALongLongALong3
> constructor(
    parameterWithLongLongLongLongLongLongLongLongNameA: A,
    parameterWithLongLongLongLongLongLongLongLongNameB: B,
    parameterWithLongLongLongLongLongLongLongLongNameC: C
)