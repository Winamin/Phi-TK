# Add project specific ProGuard rules here.
-keep class com.sc.phi.tk.PlatformPlugin { *; }
-keep class com.sc.phi.tk.*Args { *; }
-keep class quad_native.QuadNative { *; }
-keepclasseswithmembernames class com.sc.phi.tk.PreviewActivity { native <methods>; }
-keepclassmembers class com.sc.phi.tk.PreviewActivity {
    public void setFullScreen(boolean);
    public void showKeyboard(boolean);
    public void previewFailed();
}
# You can control the set of applied configuration files using the
# proguardFiles setting in build.gradle.
#
# For more details, see
#   http://developer.android.com/guide/developing/tools/proguard.html

# If your project uses WebView with JS, uncomment the following
# and specify the fully qualified class name to the JavaScript interface
# class:
#-keepclassmembers class fqcn.of.javascript.interface.for.webview {
#   public *;
#}

# Uncomment this to preserve the line number information for
# debugging stack traces.
#-keepattributes SourceFile,LineNumberTable

# If you keep the line number information, uncomment this to
# hide the original source file name.
#-renamesourcefileattribute SourceFile
