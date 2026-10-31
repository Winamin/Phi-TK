package com.sc.phi.tk

import android.os.Bundle
import android.os.Looper
import android.view.KeyEvent
import android.view.MotionEvent
import android.view.SurfaceHolder
import android.view.SurfaceView
import android.view.WindowManager
import android.view.inputmethod.InputMethodManager
import android.widget.Toast
import androidx.activity.OnBackPressedCallback
import androidx.appcompat.app.AppCompatActivity
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import androidx.core.view.WindowInsetsControllerCompat
import quad_native.QuadNative
import java.io.File

class PreviewActivity : AppCompatActivity(), SurfaceHolder.Callback {
    private lateinit var surface: SurfaceView
    private var sessionFile: File? = null
    private var started = false
    private external fun preparePreview(json: String, cacheDir: String): Boolean
    private external fun releasePreview()

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        try {
            val session = File(requireNotNull(intent.getStringExtra("session"))).canonicalFile
            require(session.parentFile == cacheDir.canonicalFile && session.name.startsWith("preview-"))
            sessionFile = session
            val json = session.readText()
            check(preparePreview(json, cacheDir.absolutePath))
            if (!contextInitialized) {
                QuadNative.initializeContext(applicationContext)
                contextInitialized = true
            }
            surface = SurfaceView(this).apply {
                holder.addCallback(this@PreviewActivity)
                isFocusableInTouchMode = true
                setOnTouchListener { _, event -> forwardTouch(event) }
                setOnKeyListener { _, key, event ->
                    if (key == KeyEvent.KEYCODE_BACK || key == KeyEvent.KEYCODE_VOLUME_UP || key == KeyEvent.KEYCODE_VOLUME_DOWN) false
                    else {
                        if (event.action == KeyEvent.ACTION_DOWN) QuadNative.surfaceOnKeyDown(key)
                        if (event.action == KeyEvent.ACTION_UP) QuadNative.surfaceOnKeyUp(key)
                        true
                    }
                }
            }
            QuadNative.activityOnCreate(this)
            started = true
            setContentView(surface)
            surface.requestFocus()
            window.addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON)
            setFullScreen(true)
            onBackPressedDispatcher.addCallback(this, object : OnBackPressedCallback(true) {
                override fun handleOnBackPressed() = finish()
            })
        } catch (error: Exception) {
            Toast.makeText(this, R.string.preview_failed, Toast.LENGTH_LONG).show()
            finish()
        }
    }

    private fun forwardTouch(event: MotionEvent): Boolean {
        val phase = when (event.actionMasked) {
            MotionEvent.ACTION_MOVE -> 0
            MotionEvent.ACTION_UP, MotionEvent.ACTION_POINTER_UP -> 1
            MotionEvent.ACTION_DOWN, MotionEvent.ACTION_POINTER_DOWN -> 2
            MotionEvent.ACTION_CANCEL -> 3
            else -> return false
        }
        val indices = if (phase == 0 || phase == 3) 0 until event.pointerCount else event.actionIndex..event.actionIndex
        for (index in indices) QuadNative.surfaceOnTouch(event.getPointerId(index), phase, event.getX(index), event.getY(index), event.eventTime)
        return true
    }

    override fun surfaceCreated(holder: SurfaceHolder) { if (started) QuadNative.surfaceOnSurfaceCreated(holder.surface) }
    override fun surfaceChanged(holder: SurfaceHolder, format: Int, width: Int, height: Int) {
        if (started) QuadNative.surfaceOnSurfaceChanged(holder.surface, width, height)
    }
    override fun surfaceDestroyed(holder: SurfaceHolder) { if (started) QuadNative.surfaceOnSurfaceDestroyed(holder.surface) }
    override fun onResume() { super.onResume(); if (started) QuadNative.activityOnResume() }
    override fun onPause() { if (started) QuadNative.activityOnPause(); super.onPause() }
    override fun onDestroy() {
        if (started) { QuadNative.activityOnDestroy(); started = false }
        releasePreview()
        // Keep the session across Activity recreation; remove it when the user leaves.
        if (isFinishing) sessionFile?.delete()
        super.onDestroy()
    }

    override fun finish() {
        if (Looper.myLooper() == Looper.getMainLooper()) super.finish()
        else runOnUiThread { super.finish() }
    }

    // Called by the miniquad Android backend.
    fun previewFailed() = runOnUiThread {
        Toast.makeText(this, R.string.preview_failed, Toast.LENGTH_LONG).show()
    }

    fun setFullScreen(fullscreen: Boolean) = runOnUiThread {
        WindowCompat.setDecorFitsSystemWindows(window, !fullscreen)
        WindowInsetsControllerCompat(window, window.decorView).apply {
            systemBarsBehavior = WindowInsetsControllerCompat.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE
            if (fullscreen) hide(WindowInsetsCompat.Type.systemBars()) else show(WindowInsetsCompat.Type.systemBars())
        }
    }

    fun showKeyboard(show: Boolean) = runOnUiThread {
        val keyboard = getSystemService(INPUT_METHOD_SERVICE) as InputMethodManager
        if (show) keyboard.showSoftInput(surface, InputMethodManager.SHOW_IMPLICIT)
        else keyboard.hideSoftInputFromWindow(surface.windowToken, 0)
    }

    companion object {
        private var contextInitialized = false
        init { System.loadLibrary("app_lib") }
    }
}
