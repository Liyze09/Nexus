package io.github.liyze09.nexus.model.block;

import java.util.ArrayList;
import java.util.Optional;

import io.github.liyze09.nexus.utils.VkPos;
import net.minecraft.world.phys.AABB;

public interface Model {
    default Optional<Mesh> getMesh(VisibleFaces visibleFaces, VkPos blockPos) {
        return Optional.empty();
    }
    default ArrayList<AABB> getAABB(VisibleFaces faces, VkPos pos, float depth) {
        return new ArrayList<>();
    }
    class Nothing implements Model {
    }
}
