package io.github.liyze09.nexus;

import net.fabricmc.api.ClientModInitializer;

public class NexusClientMain implements ClientModInitializer {
	public static Configuration config = new Configuration();
	@Override
	public void onInitializeClient() {
		
	}
	static {
		System.loadLibrary("nexus");
	}

	public static native long initNative();
	public static native long refresh(long ctx);
    public static native void close(long ctx);
    public static native void resize(long ctx, int width, int height);
    public static native long getGLReady(long ctx);
    public static native long getGLComplete(long ctx);
}