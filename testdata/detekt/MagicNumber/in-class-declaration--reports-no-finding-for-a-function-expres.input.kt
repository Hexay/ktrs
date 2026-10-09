import java.time.Duration

class SomeClassWithDefault constructor(val defaultValue: Duration = 10.toDuration(DurationUnit.MILLISECONDS))