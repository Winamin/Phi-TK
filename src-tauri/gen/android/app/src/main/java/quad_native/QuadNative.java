package quad_native;

import android.view.Surface;

/** JNI entry points exported by the miniquad Android backend. */
public final class QuadNative {
    private QuadNative() {}
    public static native void initializeContext(Object context);
    public static native void activityOnCreate(Object activity);
    public static native void activityOnResume();
    public static native void activityOnPause();
    public static native void activityOnDestroy();
    public static native void surfaceOnSurfaceCreated(Surface surface);
    public static native void surfaceOnSurfaceDestroyed(Surface surface);
    public static native void surfaceOnSurfaceChanged(Surface surface, int width, int height);
    public static native void surfaceOnTouch(int id, int phase, float x, float y, long time);
    public static native void surfaceOnKeyDown(int keycode);
    public static native void surfaceOnKeyUp(int keycode);
}
