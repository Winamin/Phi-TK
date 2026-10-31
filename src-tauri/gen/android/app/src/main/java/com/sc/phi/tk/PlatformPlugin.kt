package com.sc.phi.tk

import android.Manifest
import android.app.Activity
import android.content.ContentValues
import android.content.Intent
import android.net.Uri
import android.media.MediaScannerConnection
import android.os.Build
import android.os.Environment
import android.provider.DocumentsContract
import android.provider.MediaStore
import androidx.activity.result.ActivityResult
import androidx.documentfile.provider.DocumentFile
import app.tauri.PermissionState
import app.tauri.annotation.ActivityCallback
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.Permission
import app.tauri.annotation.PermissionCallback
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSArray
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import java.io.File
import java.security.MessageDigest
import java.util.UUID
import java.util.concurrent.Executors

@InvokeArg class PickChartsArgs { var directory = false; var multiple = false }
@InvokeArg class ImportPathArgs { lateinit var path: String }
@InvokeArg class PublishVideoArgs { lateinit var source: String; lateinit var name: String }
@InvokeArg class PreviewArgs { lateinit var params: String }
@InvokeArg class ViewVideoArgs { var uri: String? = null }

@TauriPlugin(permissions = [Permission(strings = [Manifest.permission.WRITE_EXTERNAL_STORAGE], alias = "legacyStorage")])
class PlatformPlugin(private val activity: Activity) : Plugin(activity) {
    private val io = Executors.newSingleThreadExecutor()
    private val resolver get() = activity.contentResolver

    private fun work(invoke: Invoke, block: () -> JSObject) {
        io.execute {
            try { invoke.resolve(block()) }
            catch (error: Exception) { invoke.reject(error.message ?: "Unable to access this file") }
        }
    }

    @Command
    fun pickCharts(invoke: Invoke) {
        val args = invoke.parseArgs(PickChartsArgs::class.java)
        val intent = if (args.directory) Intent(Intent.ACTION_OPEN_DOCUMENT_TREE) else {
            Intent(Intent.ACTION_OPEN_DOCUMENT).apply {
                addCategory(Intent.CATEGORY_OPENABLE)
                // PEZ archives do not have a registered MIME type on every provider.
                type = "*/*"
                putExtra(Intent.EXTRA_ALLOW_MULTIPLE, args.multiple)
            }
        }
        intent.addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION or Intent.FLAG_GRANT_PERSISTABLE_URI_PERMISSION)
        startActivityForResult(invoke, intent, "chartsPicked")
    }

    @ActivityCallback
    fun chartsPicked(invoke: Invoke, result: ActivityResult) {
        if (result.resultCode != Activity.RESULT_OK) {
            invoke.resolve(JSObject().put("paths", JSArray()))
            return
        }
        val data = result.data
        val uris = data?.clipData?.let { clip -> (0 until clip.itemCount).map { clip.getItemAt(it).uri } }
            ?: listOfNotNull(data?.data)
        work(invoke) {
            val paths = uris.map { uri ->
                // The imported copy also works with providers that cannot persist a grant.
                try { resolver.takePersistableUriPermission(uri, Intent.FLAG_GRANT_READ_URI_PERMISSION) }
                catch (_: SecurityException) { }
                importUri(uri).absolutePath
            }
            JSObject().put("paths", JSArray(paths))
        }
    }

    @Command
    fun importPath(invoke: Invoke) = work(invoke) {
        val uri = Uri.parse(invoke.parseArgs(ImportPathArgs::class.java).path)
        val file = when (uri.scheme) {
            "content" -> importUri(uri)
            "file" -> File(requireNotNull(uri.path))
            else -> File(uri.toString())
        }
        require(file.exists()) { "The chart is no longer available. Please select it again." }
        JSObject().put("path", file.absolutePath)
    }

    private fun importUri(uri: Uri): File {
        val key = MessageDigest.getInstance("SHA-256").digest(uri.toString().toByteArray())
            .joinToString("") { "%02x".format(it) }
        val root = File(activity.filesDir, "charts").apply { mkdirs() }
        val tree = DocumentsContract.isTreeUri(uri)
        val target = File(root, if (tree) key else "$key.zip")
        if (target.exists()) return target
        val temporary = File(root, "$key-${UUID.randomUUID()}.part")
        try {
            if (tree) {
                val document = requireNotNull(DocumentFile.fromTreeUri(activity, uri))
                copyDirectory(document, temporary)
            } else {
                resolver.openInputStream(uri).use { input ->
                    requireNotNull(input) { "Cannot read the selected chart" }
                    temporary.outputStream().use { input.copyTo(it) }
                }
                require(temporary.length() > 0) { "The selected chart is empty" }
            }
            check(temporary.renameTo(target)) { "Cannot save the imported chart" }
            return target
        } finally {
            temporary.deleteRecursively()
        }
    }

    private fun copyDirectory(document: DocumentFile, destination: File, depth: Int = 0) {
        require(depth < 64 && document.canRead()) { "Cannot read this chart folder" }
        check(destination.mkdirs()) { "Cannot create the imported chart folder" }
        for (child in document.listFiles()) {
            val name = requireNotNull(child.name)
            require(name != "." && name != ".." && '/' !in name && '\\' !in name)
            val target = File(destination, name)
            if (child.isDirectory) copyDirectory(child, target, depth + 1)
            else resolver.openInputStream(child.uri).use { input ->
                requireNotNull(input) { "Cannot read $name" }
                target.outputStream().use { input.copyTo(it) }
            }
        }
    }

    @Command
    fun publishVideo(invoke: Invoke) {
        if (Build.VERSION.SDK_INT < 29 && getPermissionState("legacyStorage") != PermissionState.GRANTED) {
            requestPermissionForAlias("legacyStorage", invoke, "storageGranted")
            return
        }
        publish(invoke)
    }

    @PermissionCallback
    fun storageGranted(invoke: Invoke) {
        if (getPermissionState("legacyStorage") == PermissionState.GRANTED) publish(invoke)
        else invoke.reject("Storage permission is required to save videos on Android 9 and earlier")
    }

    private fun publish(invoke: Invoke) = work(invoke) {
        val args = invoke.parseArgs(PublishVideoArgs::class.java)
        val source = File(args.source)
        require(source.isFile && source.length() > 0) { "Rendered video is missing or empty" }
        require(File(args.name).name == args.name) { "Invalid video name" }
        val values = ContentValues().apply {
            put(MediaStore.Video.Media.DISPLAY_NAME, args.name)
            put(MediaStore.Video.Media.MIME_TYPE, "video/mp4")
            if (Build.VERSION.SDK_INT >= 29) {
                put(MediaStore.Video.Media.RELATIVE_PATH, Environment.DIRECTORY_MOVIES + "/Phi-TK")
                put(MediaStore.Video.Media.IS_PENDING, 1)
            } else {
                val directory = File(Environment.getExternalStoragePublicDirectory(Environment.DIRECTORY_MOVIES), "Phi-TK")
                check(directory.isDirectory || directory.mkdirs())
                val target = File(directory, "${UUID.randomUUID()}-${args.name}")
                @Suppress("DEPRECATION")
                put(MediaStore.Video.Media.DATA, target.absolutePath)
            }
        }
        val collection = if (Build.VERSION.SDK_INT >= 29)
            MediaStore.Video.Media.getContentUri(MediaStore.VOLUME_EXTERNAL_PRIMARY)
        else MediaStore.Video.Media.EXTERNAL_CONTENT_URI
        val uri = checkNotNull(resolver.insert(collection, values)) { "Cannot create video in the media library" }
        try {
            resolver.openOutputStream(uri, "w").use { output ->
                checkNotNull(output) { "Cannot write to the media library" }
                source.inputStream().use { it.copyTo(output) }
            }
            if (Build.VERSION.SDK_INT >= 29) {
                val ready = ContentValues().apply { put(MediaStore.Video.Media.IS_PENDING, 0) }
                check(resolver.update(uri, ready, null, null) == 1) { "Cannot publish video to the gallery" }
            } else {
                @Suppress("DEPRECATION")
                val path = requireNotNull(values.getAsString(MediaStore.Video.Media.DATA))
                MediaScannerConnection.scanFile(activity, arrayOf(path), arrayOf("video/mp4"), null)
            }
            JSObject().put("uri", uri.toString())
        } catch (error: Exception) {
            resolver.delete(uri, null, null)
            throw error
        }
    }

    @Command
    fun viewVideo(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(ViewVideoArgs::class.java)
            val intent = if (args.uri != null) {
                val uri = Uri.parse(args.uri)
                require(uri.scheme == "content" && uri.authority == "media")
                Intent(Intent.ACTION_VIEW).setDataAndType(uri, "video/mp4")
                    .addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
            } else Intent(Intent.ACTION_MAIN).addCategory(Intent.CATEGORY_APP_GALLERY)
            activity.startActivity(intent)
            invoke.resolve()
        } catch (error: Exception) {
            invoke.reject("Open your gallery and look for the Phi-TK album in Movies.")
        }
    }

    @Command
    fun launchPreview(invoke: Invoke) {
        val args = invoke.parseArgs(PreviewArgs::class.java)
        io.execute {
            var session: File? = null
            try {
                session = File.createTempFile("preview-", ".json", activity.cacheDir)
                session.writeText(args.params)
                val path = session.absolutePath
                activity.runOnUiThread {
                    try {
                        activity.startActivity(Intent(activity, PreviewActivity::class.java).putExtra("session", path))
                        invoke.resolve()
                    } catch (error: Exception) {
                        File(path).delete()
                        invoke.reject(error.message ?: "Cannot open preview")
                    }
                }
            } catch (error: Exception) {
                session?.delete()
                invoke.reject(error.message ?: "Cannot prepare preview")
            }
        }
    }
}
