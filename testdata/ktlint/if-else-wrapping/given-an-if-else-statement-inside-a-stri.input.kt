fun foo() {
    logger.log(
        "<-- ${if (true)
            ""
        else
            ' ' +
                "bar"}",
    )
}