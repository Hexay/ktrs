package io.github.hexay.ktrs.gradle.ktlint

import java.io.File
import java.io.ObjectInputStream
import java.io.ObjectOutputStream
import java.io.Serializable
import java.security.MessageDigest

/**
 * ktlint-gradle's `KtLintWorkAction.FormatTaskSnapshot`: the files the last format run changed, with
 * their content hash before formatting, so a file restored to that content makes the task run again.
 */
internal class FormatTaskSnapshot(val formattedSources: Map<File, ByteArray>) : Serializable {

    companion object {
        private const val serialVersionUID = 1L

        fun readFromFile(snapshotFile: File): FormatTaskSnapshot =
            if (snapshotFile.exists()) {
                ObjectInputStream(snapshotFile.inputStream().buffered()).use { it.readObject() as FormatTaskSnapshot }
            } else {
                FormatTaskSnapshot(emptyMap())
            }

        fun writeIntoFile(snapshotFile: File, formatSnapshot: FormatTaskSnapshot) {
            snapshotFile.parentFile.mkdirs()
            ObjectOutputStream(snapshotFile.outputStream().buffered()).use { it.writeObject(formatSnapshot) }
        }

        fun contentHash(file: File): ByteArray = MessageDigest.getInstance("MD5").digest(file.readBytes())
    }
}
