package io.github.liyze09.nexus.utils;

import net.minecraft.core.BlockPos;
import net.minecraft.core.Vec3i;

public class VkPos extends BlockPos {
    public VkPos(int x, int y, int z) {
        super(x, y, z);
    }
    public VkPos(BlockPos pos) {
        super(pos.getX(), pos.getY(), -pos.getZ());
    }
    public VkPos(Vec3i pos) {
        super(pos);
    }
}
