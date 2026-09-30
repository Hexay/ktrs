// Max line length marker:          #
fun foo() {
    logger.info {
        "fooooooooooooooooooooooooooo"
    }
    logger.info {
        "foo" + "oooooooooooooooooooo"
    }
}