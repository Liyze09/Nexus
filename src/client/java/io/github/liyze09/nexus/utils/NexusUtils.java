package io.github.liyze09.nexus.utils;

import net.minecraft.core.BlockPos;
import net.minecraft.core.Direction;
import net.minecraft.world.level.chunk.LevelChunk;
import net.minecraft.world.phys.AABB;
import net.minecraft.world.phys.shapes.VoxelShape;
import org.jetbrains.annotations.Contract;
import org.jetbrains.annotations.NotNull;

public final class NexusUtils {
    public NexusUtils() {
        throw new UnsupportedOperationException("Utility class");
    }

    @Contract("_, _ -> new")
    public static @NotNull AABB getInflatedAABB(@NotNull VoxelShape shape, double d) {
        return new AABB(
                shape.min(Direction.Axis.X) - d,
                shape.min(Direction.Axis.Y) - d,
                shape.min(Direction.Axis.Z) - d,
                shape.max(Direction.Axis.X) + d,
                shape.max(Direction.Axis.Y) + d,
                shape.max(Direction.Axis.Z) + d
        );
    }

    public static boolean inArea(@NotNull BlockPos pos, @NotNull BlockPos min, @NotNull BlockPos max) {
        return pos.getX() >= min.getX() && pos.getX() <= max.getX()
                && pos.getY() >= min.getY() && pos.getY() <= max.getY()
                && pos.getZ() >= min.getZ() && pos.getZ() <= max.getZ();
    }

    public static boolean inChunk(@NotNull BlockPos pos, @NotNull LevelChunk chunk) {
        int minX = chunk.getPos().getMinBlockX();
        int minZ = chunk.getPos().getMinBlockZ();
        int maxX = chunk.getPos().getMaxBlockX();
        int maxZ = chunk.getPos().getMaxBlockZ();
        int minY = chunk.getMinY();
        int maxY = chunk.getMaxY();
        return pos.getX() >= minX && pos.getX() <= maxX
                && pos.getY() >= minY && pos.getY() <= maxY
                && pos.getZ() >= minZ && pos.getZ() <= maxZ;
    }

    public static BlockPos asChunkPos(@NotNull BlockPos pos) {
        return new BlockPos(
                pos.getX() & 15,
                pos.getY(),
                pos.getZ() & 15
        );
    }

    public static VkPos transformToVk(@NotNull BlockPos pos) {
        return new VkPos(
                pos
        );
    }
}
