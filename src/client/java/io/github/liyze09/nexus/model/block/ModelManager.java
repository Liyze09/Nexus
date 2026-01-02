package io.github.liyze09.nexus.model.block;

import net.minecraft.world.level.block.state.BlockState;
import org.jetbrains.annotations.NotNull;

import java.util.HashMap;

public class ModelManager {
    private static final ModelManager instance = new ModelManager();
    public static ModelManager getInstance() {
        return instance;
    }
    public HashMap<BlockState, Model> map = new HashMap<>();
    private ModelManager() {
        // TODO: load models
    }
    @NotNull
    public Model getModel(BlockState state) {
        return map.getOrDefault(state, BoundingBox.INSTANCE);
    }
}
