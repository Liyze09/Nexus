package io.github.liyze09.nexus;

import net.fabricmc.api.ClientModInitializer;

public class NexusClientMain implements ClientModInitializer {
    public static Configuration config = new Configuration();

    static {
        System.loadLibrary("nexus");
    }

    public static native long initNative();

    public static native void render(long ctx);

    public static native void close(long ctx);

    public static native long getTextureSize(long ctx);

    public static native void resize(long ctx, int width, int height);

    public static native long getGLReady(long ctx);

    public static native long getGLComplete(long ctx);

    public static native long getGLTexture(long ctx);

    public static native long acquireVulkanTexture(long ctx, int width, int height, int mipLevels);

    public static native long getVulkanTextureSize(long ctx, long handle);

    @Override
    public void onInitializeClient() {

    }
}