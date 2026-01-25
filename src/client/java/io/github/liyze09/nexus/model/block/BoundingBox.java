package io.github.liyze09.nexus.model.block;

import io.github.liyze09.nexus.utils.VkPos;
import net.minecraft.world.phys.AABB;

import java.util.ArrayList;

public class BoundingBox implements Model {
    public static final BoundingBox INSTANCE = new BoundingBox();

    @Override
    public ArrayList<AABB> getAABB(VisibleFaces faces, VkPos pos, float depth) {
        var aabbs = new ArrayList<AABB>();
        if (faces.north) {
            aabbs.add(new AABB(
                    pos.getX(),
                    pos.getY(),
                    pos.getZ() + 1,
                    pos.getX() + 1,
                    pos.getY() + 1,
                    pos.getZ() + 1 + depth
            ));
        }
        if (faces.south) {
            aabbs.add(new AABB(
                    pos.getX(),
                    pos.getY(),
                    pos.getZ() - depth,
                    pos.getX() + 1,
                    pos.getY() + 1,
                    pos.getZ()
            ));
        }
        if (faces.west) {
            aabbs.add(new AABB(
                    pos.getX() - depth,
                    pos.getY(),
                    pos.getZ(),
                    pos.getX(),
                    pos.getY() + 1,
                    pos.getZ() + 1
            ));
        }
        if (faces.east) {
            aabbs.add(new AABB(
                    pos.getX() + 1,
                    pos.getY(),
                    pos.getZ(),
                    pos.getX() + 1 + depth,
                    pos.getY() + 1,
                    pos.getZ() + 1
            ));
        }
        if (faces.up) {
            aabbs.add(new AABB(
                    pos.getX(),
                    pos.getY() + 1,
                    pos.getZ(),
                    pos.getX() + 1,
                    pos.getY() + 1 + depth,
                    pos.getZ() + 1
            ));
        }
        if (faces.down) {
            aabbs.add(new AABB(
                    pos.getX(),
                    pos.getY() - depth,
                    pos.getZ(),
                    pos.getX() + 1,
                    pos.getY(),
                    pos.getZ() + 1
            ));
        }
        return aabbs;
    }
}
