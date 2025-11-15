package io.github.liyze09.nexus;

import io.github.liyze09.nexus.chunk.NexusChunkBuilder;
import net.fabricmc.api.ClientModInitializer;

public class NexusClientMain implements ClientModInitializer {
	public static NexusChunkBuilder builder;
	@Override
	public void onInitializeClient() {

		builder = new NexusChunkBuilder();
		initNative();
	}
	static {
		System.loadLibrary("nexus");
	}

	public static native long initNative();
	public static native long render(long ctx);
    public static native void close(long ctx);
}