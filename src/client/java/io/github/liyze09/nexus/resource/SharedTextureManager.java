package io.github.liyze09.nexus.resource;

import com.google.common.collect.ImmutableList;
import com.mojang.blaze3d.textures.GpuTexture;
import it.unimi.dsi.fastutil.objects.Object2LongMap;
import it.unimi.dsi.fastutil.objects.Object2LongOpenHashMap;
import net.minecraft.resources.Identifier;

import java.util.concurrent.ConcurrentHashMap;

public class SharedTextureManager {
    public final ConcurrentHashMap<Identifier, GpuTexture> textures = new ConcurrentHashMap<>();
    @SuppressWarnings("null")
    public final ImmutableList<Identifier> textureManagedByVulkan = ImmutableList.of(
            Identifier.withDefaultNamespace("textures/atlas/blocks.png")
    );
    public void addTexture(Identifier identifier, GpuTexture texture) {
        textures.put(identifier, texture);
    }

    public Object2LongMap<Identifier> genExportedTextureIds() {
        var ret = new Object2LongOpenHashMap<Identifier>();
        textures.forEach((identifier, texture) ->
                ret.put(identifier,
                        ((ExternalGlTexture) texture).handle
                )
        );
        return ret;
    }
}
