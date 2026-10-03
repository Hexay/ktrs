package io.github.hexay.ktrs;

/** Builds a {@code ktrs serve} request header (protocol: crates/ktrs-cli/src/serve.rs). */
final class RequestHeader {
    private final StringBuilder text = new StringBuilder();

    /** Adds {@code key=value}; a null value adds nothing. */
    RequestHeader line(String key, Object value) {
        if (value == null) {
            return this;
        }
        String string = value.toString();
        if (string.indexOf('\n') >= 0 || string.indexOf('\r') >= 0) {
            throw new IllegalArgumentException(key + " contains a line break: " + string);
        }
        text.append(key).append('=').append(string).append('\n');
        return this;
    }

    @Override
    public String toString() {
        return text.toString();
    }
}
