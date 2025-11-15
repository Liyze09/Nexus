package io.github.liyze09.nexus.chunk;

import net.minecraft.core.BlockPos;

public class NexusChunkBuilder {

    public static class NexusChunk {

    }
    public NexusChunk getNexusChunk(BlockPos pos) {
        return new NexusChunk();
    }
    public boolean isSectionBuilt(BlockPos pos) {
        return true;
    }
    public void rebuild(int x, int y, int z, boolean important) {}
}
