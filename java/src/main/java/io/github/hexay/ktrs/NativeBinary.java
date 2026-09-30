package io.github.hexay.ktrs;

import java.io.IOException;
import java.io.InputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.nio.file.StandardCopyOption;
import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.util.Arrays;
import java.util.Locale;

/**
 * Finds the ktrs executable: {@code -Dktrs.executable}, else the binary bundled in this jar for the
 * current platform, extracted once to a cache directory keyed by its hash.
 */
final class NativeBinary {
    private static volatile Path bundled;

    private NativeBinary() {}

    static Path locate() {
        String override = System.getProperty("ktrs.executable");
        if (override != null) {
            return Paths.get(override);
        }
        Path path = bundled;
        if (path == null) {
            synchronized (NativeBinary.class) {
                path = bundled;
                if (path == null) {
                    path = extractBundled();
                    bundled = path;
                }
            }
        }
        return path;
    }

    private static Path extractBundled() {
        String platform = platform();
        String name = platform.startsWith("windows") ? "ktrs.exe" : "ktrs";
        byte[] bytes;
        try (InputStream stream = NativeBinary.class.getResourceAsStream("native/" + platform + "/" + name)) {
            if (stream == null) {
                throw new KtrsException("this ktrs jar has no binary for " + platform
                        + "; install ktrs and pass its path to Ktrs.create(Path) or -Dktrs.executable");
            }
            bytes = stream.readAllBytes();
            return extract(bytes, cacheDir().resolve(sha256(bytes).substring(0, 16)).resolve(name));
        } catch (IOException e) {
            throw new KtrsException("could not extract the bundled ktrs binary: " + e.getMessage(), e);
        }
    }

    /** {@code <os>-<arch>}: linux, macos or windows; x86_64 or aarch64. */
    static String platform() {
        String os = System.getProperty("os.name").toLowerCase(Locale.ROOT);
        String arch = System.getProperty("os.arch").toLowerCase(Locale.ROOT);
        String osName = os.startsWith("windows") ? "windows" : os.startsWith("mac") ? "macos" : "linux";
        String archName = arch.equals("aarch64") || arch.equals("arm64") ? "aarch64" : "x86_64";
        return osName + "-" + archName;
    }

    private static Path extract(byte[] bytes, Path target) throws IOException {
        if (Files.isRegularFile(target) && Arrays.equals(sha256Bytes(Files.readAllBytes(target)), sha256Bytes(bytes))) {
            return target;
        }
        Files.createDirectories(target.getParent());
        Path temp = Files.createTempFile(target.getParent(), "ktrs", ".tmp");
        try {
            Files.write(temp, bytes);
            if (!temp.toFile().setExecutable(true)) {
                throw new IOException("cannot make " + temp + " executable");
            }
            Files.move(temp, target, StandardCopyOption.ATOMIC_MOVE, StandardCopyOption.REPLACE_EXISTING);
        } catch (IOException e) {
            // Another JVM may have won the race (Windows refuses to replace a running executable).
            if (!Files.isRegularFile(target)) {
                throw e;
            }
        } finally {
            Files.deleteIfExists(temp);
        }
        return target;
    }

    private static Path cacheDir() {
        String override = System.getProperty("ktrs.cache.dir");
        if (override != null) {
            return Paths.get(override);
        }
        String home = System.getProperty("user.home");
        String localAppData = System.getenv("LOCALAPPDATA");
        String xdg = System.getenv("XDG_CACHE_HOME");
        switch (platform().substring(0, platform().indexOf('-'))) {
            case "windows":
                return Paths.get(localAppData != null ? localAppData : home, "ktrs", "cache");
            case "macos":
                return Paths.get(home, "Library", "Caches", "ktrs");
            default:
                return xdg != null ? Paths.get(xdg, "ktrs") : Paths.get(home, ".cache", "ktrs");
        }
    }

    private static String sha256(byte[] bytes) {
        StringBuilder hex = new StringBuilder();
        for (byte b : sha256Bytes(bytes)) {
            hex.append(String.format("%02x", b));
        }
        return hex.toString();
    }

    private static byte[] sha256Bytes(byte[] bytes) {
        try {
            return MessageDigest.getInstance("SHA-256").digest(bytes);
        } catch (NoSuchAlgorithmException e) {
            throw new AssertionError(e);
        }
    }
}
