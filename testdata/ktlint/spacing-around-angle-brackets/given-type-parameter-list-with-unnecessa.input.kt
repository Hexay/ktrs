public class Foo1<Bar : String> {}
public class Foo2< Bar : String> {}
public class Foo3<Bar : String > {}
public class Foo4 <Bar : String> {}
public class Foo5
    <Bar : String> {}
public class Foo6
    < Bar : String > {}
public class Foo7<
    Bar1 : String,
    Bar2 : Map<
        Int,
        List< String >
        >
    > {}
public class Foo8<

    Bar1 : String,
    Bar2 : String

> {}