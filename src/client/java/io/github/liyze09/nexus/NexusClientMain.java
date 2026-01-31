package io.github.liyze09.nexus;

import io.github.liyze09.nexus.render.NexusWorldRenderer;
import net.fabricmc.api.ClientModInitializer;
import net.minecraft.client.Minecraft;

public class NexusClientMain implements ClientModInitializer {
    public static Configuration config = new Configuration();

    public static NexusWorldRenderer getNexusRenderer() {
        if (Minecraft.getInstance().levelRenderer instanceof NexusWorldRenderer renderer)
            return renderer;
        else {
            throw new IllegalStateException("NexusWorldRenderer is not initialized");
        }
    }

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

    public static native void syncAtlas(long ctx, long textureHandle, String atlasName,
                                        String[] spriteNames,
                                        int[] spriteX, int[] spriteY,
                                        int[] spriteWidth, int[] spriteHeight,
                                        float[] spriteU0, float[] spriteV0,
                                        float[] spriteU1, float[] spriteV1);

    @Override
    public void onInitializeClient() {

    }
}