package io.github.hexay.ktrs;

/** The code could not be formatted (a syntax error, in ktfmt's wording), or the server failed. */
public class KtrsException extends RuntimeException {
    private static final long serialVersionUID = 1L;

    public KtrsException(String message) {
        super(message);
    }

    public KtrsException(String message, Throwable cause) {
        super(message, cause);
    }
}
